mod shared;
mod physics;
mod network;
mod utils;
mod components;
use crate::shared::*;

use renet::{RenetServer, ServerEvent, DefaultChannel};
use renet_netcode::NetcodeServerTransport;

use std::time::Duration;
use std::collections::HashMap;

struct Server {
    shared: Shared,
    server: RenetServer,
    transport: NetcodeServerTransport,
    client_ids: HashMap<ClientId, Entity>,
}

impl Server {
    fn create(addr: String) -> Self {
        let (server, transport) = create_server(addr);

        Self {
            shared: Shared::new(),
            server,
            transport,
            client_ids: HashMap::new(),
        }
    }

    async fn start(&mut self) {
        self.shared.load_map(TEST_MAP.to_string()).await;

        let mut update = Interval::new(Duration::from_millis(1000 / 30));
        let mut dt_accumulator = 0.0;

        loop {
            if update.tick() {
                let delta = update.delta();
                dt_accumulator += delta.as_secs_f32();

                self.network_receive(delta).await;

                while dt_accumulator >= PHYSICS_STEP {
                    self.shared.handle_physics(PHYSICS_STEP).await;
                    dt_accumulator -= PHYSICS_STEP;
                }

                self.network_send().await;
            }
        }
    }

    async fn get_random_spawn(&self) -> Vec3 {
        let mut spawn_points = Vec::new();

        for (_id, (pos, props)) in self.shared.ecs.query::<(&PointObject, &Properties)>().iter() {
            if let Some(spawn) = props.spawn {
                if spawn {
                    spawn_points.push(pos.0);
                }
            }
        }

        *spawn_points.choose().unwrap_or_else(|| &Vec3::ZERO)
    }

    fn spawn_gibs(&mut self, target: Vec3) {
        for x in 0..2 {
            for z in 0..2 {
                let x = (x * 2 - 1) as f32;
                let z = (z * 2 - 1) as f32;
                self.shared.ecs.spawn({
                    let (rig, col) = self.shared.physics.spawn_cube(
                        target + vec3(x / 2.0, 0.0, z / 2.0),
                        vec3(0.5, 0.5, 0.5)
                    );
                    self.shared.physics.get_rig_mut(rig).set_linvel(vector![x, 0.0, z], false);
                    (rig, col)
                });
            }
        }

        self.shared.ecs.spawn({
            let (rig, col) = self.shared.physics.spawn_cube(
                target + vec3(0.0, 1.0, 0.0),
                vec3(0.5, 0.5, 0.5)
            );
            self.shared.physics.get_rig_mut(rig).set_linvel(vector![0.0, 10.0, 0.0], false);
            (rig, col)
        });
    }

    async fn handle_msg(&mut self, cli_id: ClientId, msg: ClientMessage) {
        let id = self.client_ids[&cli_id];
        match msg {
            ClientMessage::SetMoveState(state) => {
                if let Ok(mut player) = self.shared.ecs.get::<&mut Player>(id) {
                    let jumping = player.moves.jump;
                    player.moves = state;
                    player.moves.jump = jumping || player.moves.jump;
                }
            }
            ClientMessage::SetYaw(yaw) => {
                todo!()
                // if let Ok(mut obj) = self.shared.ecs.get::<&mut PhysicsObject>(id) {
                //     obj.set_yaw(yaw);
                // }
            }
            ClientMessage::Shot(shot_id) => {
                if let Some(target) = {
                    if let Ok((handle, player)) = self.shared.ecs.query_one_mut::<(&RigidBodyHandle, &mut Player)>(shot_id) {
                        player.hurt(20);
                        if player.dead() {
                            let obj = self.shared.physics.get_rig_mut(*handle);
                            let old_pos = conv_vec_2(*obj.translation());
                            obj.set_translation(vector![0.0, 60.0, 0.0], true);
                            player.reset_hp();
                            Some(old_pos)
                        } else { None }
                    } else { None }
                } {
                    self.spawn_gibs(target);
                }
            },
        }
    }

    async fn network_receive(&mut self, duration: Duration) {
        self.server.update(duration);
        self.transport.update(duration, &mut self.server).unwrap();

        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("{} connected", client_id);
                    self.server.send_message(client_id, DefaultChannel::ReliableUnordered, serialize(
                        vec![
                            ServerMessage::AssignId({
                                let (rig_handle, col_handle) = self.shared.physics.spawn_cube(
                                    self.get_random_spawn().await,
                                    vec3(1.0, 2.0, 1.0)
                                );
                                let id = self.shared.ecs.spawn((
                                    Player::new(),
                                    rig_handle,
                                    col_handle,
                                ));
                                self.client_ids.insert(client_id, id);
                                id
                            }),
                            ServerMessage::Ecs(Columns {
                                MeshWrapper: clone_column!(self, &MeshWrapper),
                                ..Columns::default()
                            }),
                            ServerMessage::PhysicsState(
                                self.shared.physics.state.rigid_body_set.clone(),
                                self.shared.physics.state.collider_set.clone()
                            )
                        ]
                    ).unwrap());
                },
                _ => {}
            }
        }

        for client in self.server.clients_id_iter().collect::<Vec<_>>().iter() {
            for channel in NET_CHANNELS {
                while let Some(ref data) = self.server.receive_message(*client, channel) {
                    match deserialize::<ClientMessages>(data) {
                        Ok(new_msgs) =>
                            for msg in new_msgs {
                                self.handle_msg(*client, msg).await;
                            }
                        Err(err) => 
                            eprintln!("{}", err)
                    }
                }
            }
        }
    }

    async fn network_send(&mut self) {
        for client in self.server.clients_id_iter().collect::<Vec<_>>().iter() {
            self.server.send_message(*client, DefaultChannel::Unreliable, serialize(
                vec![
                    ServerMessage::Ecs(Columns {
                        Player: clone_column!(self, &Player),
                        RigidBodyHandle: clone_column!(self, &RigidBodyHandle),
                        ColliderHandle: clone_column!(self, &ColliderHandle),
                        ..Columns::default()
                    }),
                    ServerMessage::PhysicsDiff(self.shared.physics.get_physics_diff())
                ]
            ).unwrap());
        }

        self.transport.send_packets(&mut self.server);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let mut server = Server::create(args.addr);
    server.start().await;
}
