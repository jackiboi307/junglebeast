use crate::*;
use renet::{RenetServer, ServerEvent, DefaultChannel};
use renet_netcode::NetcodeServerTransport;
use std::time::Duration;
use std::collections::HashMap;

fn physobj(pos: Vec3, size: Vec3) -> PhysicsObject {
    PhysicsObject::new(Cube::new(pos, size))
}

pub struct Server {
    shared: Shared,
    server: RenetServer,
    transport: NetcodeServerTransport,
    client_ids: HashMap<ClientId, Entity>,
}

impl Server {
    pub fn create(addr: String) -> Self {
        let (server, transport) = create_server(addr);

        Self {
            shared: Shared::new(),
            server,
            transport,
            client_ids: HashMap::new(),
        }
    }

    pub async fn start(&mut self) {
        self.create_map();

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

    fn create_map(&mut self) {
        self.shared.ecs.spawn((physobj(
            vec3(0.0, -1.0, 0.0),
            vec3(60.0, 2.0, 60.0)).fixed(),));
        self.shared.ecs.spawn((physobj(
            vec3(0.0, 0.5, 5.0),
            vec3(5.0, 1.0, 1.0)).fixed(),));
        self.shared.ecs.spawn((physobj(
            vec3(0.0, 2.0, -5.0),
            vec3(5.0, 4.0, 1.0)).fixed(),));
    }

    fn spawn_gibs(&mut self, target: Vec3) {
        for x in 0..2 {
            for z in 0..2 {
                let x = (x * 2 - 1) as f32;
                let z = (z * 2 - 1) as f32;
                self.shared.ecs.spawn((
                    physobj(
                        target + vec3(x / 2.0, 0.0, z / 2.0),
                        vec3(0.5, 0.5, 0.5)
                    ).vel(vec3(x, 10.0, z)),
                ));
            }
        }

        self.shared.ecs.spawn((
            physobj(
                target + vec3(0.0, 1.0, 0.0),
                vec3(0.5, 0.5, 0.5)
            ).vel(vec3(0.0, 10.0, 0.0)),
        ));
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
            },
            ClientMessage::SetRotation(rot) => {
                if let Ok(mut obj) = self.shared.ecs.get::<&mut PhysicsObject>(id) {
                    obj.cube.rot = rot;
                }
            }
            ClientMessage::Shot(shot_id) => {
                if let Some(target) = {

                    if let Ok((obj, player)) = self.shared.ecs.query_one_mut::<(&mut PhysicsObject, &mut Player)>(shot_id) {
                        player.hurt(20);
                        if player.dead() {
                            let old_pos = obj.cube.pos;
                            obj.cube.pos = vec3(0.0, 60.0, 0.0);
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
                                let id = self.shared.ecs.spawn((
                                    Player::new(),
                                    physobj(
                                        vec3(0.0, 1.0, 0.0),
                                        vec3(1.0, 2.0, 1.0)
                                    ),
                                ));
                                self.client_ids.insert(client_id, id);
                                id
                            }),
                            ServerMessage::Ecs(Columns {
                                PhysicsObject: clone_column!(self, &PhysicsObject),
                                ..Columns::default()
                            }),
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
                        PhysicsObject: self.shared.ecs.query::<&PhysicsObject>().iter()
                            .filter(|(_, obj)| !obj.fixed)
                            .map(|(id, obj)| (id, obj.clone()))
                            .collect(),
                        Player: clone_column!(self, &Player),
                    }),
                ]
            ).unwrap());
        }

        self.transport.send_packets(&mut self.server);
    }
}
