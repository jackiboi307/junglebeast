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

        let mut update   = Interval::new(Duration::from_millis(1000 / 30));

        loop {
            if update.tick() {
                let delta = update.delta();

                self.shared.handle_physics(delta.as_secs_f32(), None).await;
                self.handle_network(delta).await;
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

    async fn handle_msg(&mut self, cli_id: ClientId, msg: ClientMessage) {
        let id = self.client_ids[&cli_id];
        match msg {
            ClientMessage::PosVel(pos, vel) => {
                let obj = self.shared.ecs.query_one_mut::<&mut PhysicsObject>(id).unwrap();
                obj.cube.pos = pos;
                obj.vel = vel;
            },
            ClientMessage::Shot(shot_id) => println!("{id:?} has shot {shot_id:?}"),
        }
    }

    async fn handle_network(&mut self, duration: Duration) {
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
                                    Player {},
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
                                Player: clone_column!(self, &Player),
                            }),
                        ]
                    ).unwrap());
                },
                _ => {}
            }
        }

        let mut msgs: Vec<(ClientId, ClientMessages)> = Vec::new();

        for client in self.server.clients_id_iter().collect::<Vec<_>>().iter() {
            msgs.push((*client, Vec::new()));

            self.server.send_message(*client, DefaultChannel::Unreliable, serialize(
                vec![
                    ServerMessage::Ecs(Columns {
                        PhysicsObject: self.shared.ecs.query::<&PhysicsObject>().iter()
                            .filter(|(_, obj)| !obj.fixed)
                            .map(|(id, obj)| (id, obj.clone()))
                            .collect(),
                        ..Columns::default()
                    }),
                ]
            ).unwrap());

            for channel in NET_CHANNELS {
                while let Some(ref data) = self.server.receive_message(*client, channel) {
                    match deserialize::<ClientMessages>(data) {
                        Ok(new_msgs) =>
                            for msg in new_msgs {
                                msgs.last_mut().unwrap().1.push(msg);
                            }
                        Err(err) => 
                            eprintln!("{}", err)
                    }
                }
            }
        }

        self.transport.send_packets(&mut self.server);

        for (id, msgs) in msgs {
            for msg in msgs {
                self.handle_msg(id, msg).await;
            }
        }
    }
}
