use crate::*;
use renet::{RenetServer, ServerEvent, DefaultChannel};
use renet_netcode::NetcodeServerTransport;
use std::time::Duration;

pub struct Server {
    shared: Shared,
    server: RenetServer,
    transport: NetcodeServerTransport,
}

impl Server {
    pub fn create(addr: String) -> Self {
        let (server, transport) = create_server(addr);

        Self {
            shared: Shared::new(),
            server,
            transport,
        }
    }

    pub async fn start(&mut self) {
        use std::time::{Duration, Instant};
        use tokio::time::sleep;

        self.create_map();

        let mut update   = Interval::new(Duration::from_millis(1000 / 30));
        let mut net_send = Interval::new(Duration::from_millis(200));
        let mut net_recv = Interval::new(Duration::from_millis(200));

        loop {
            if update.tick() {
                let delta = update.delta();

                self.shared.handle_physics(delta.as_secs_f32(), None).await;
                self.handle_network(delta, net_send.tick(), net_recv.tick()).await;
            }
        }
    }

    fn create_map(&mut self) {
        // self.ecs.spawn((physobj(
        //     vec3(0.0, 20.0, 0.0),
        //     vec3(1.0, 1.0, 1.0)),));
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

    async fn handle_msg(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::Shared(msg) => {
                match msg {
                    SharedMessage::Ecs {
                        PhysicsObject,
                    } => {
                        for (id, obj) in PhysicsObject {
                            if self.shared.ecs.entity(id).is_err() {
                                self.shared.ecs.spawn_at(id, ());
                            }

                            self.shared.ecs.insert(id, (obj,)).unwrap();
                        }
                    }
                }
            }
        }
    }

    async fn handle_network(&mut self, duration: Duration, send: bool, recv: bool) {
        self.server.update(duration);
        self.transport.update(duration, &mut self.server).unwrap();

        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("{} connected", client_id);
                    self.server.send_message(client_id, DefaultChannel::ReliableUnordered, serialize(
                        vec![
                            ServerMessage::AssignId(
                                self.shared.ecs.spawn((physobj(
                                    vec3(0.0, 1.0, 0.0),
                                    vec3(1.0, 2.0, 1.0)),
                                ))
                            ),
                            ServerMessage::Shared(SharedMessage::Ecs {
                                PhysicsObject: self.shared.ecs.query::<&PhysicsObject>().iter()
                                    .map(|(id, obj)| (id, obj.clone()))
                                    .collect()
                            }),
                        ]
                    ).unwrap());
                },
                _ => {}
            }
        }

        let mut msgs: ClientMessages = Vec::new();

        for client in self.server.clients_id_iter().collect::<Vec<_>>().iter() {
            if send {
                self.server.send_message(*client, DefaultChannel::Unreliable, serialize(
                    vec![
                        ServerMessage::Shared(SharedMessage::Ecs {
                            PhysicsObject: self.shared.ecs.query::<&PhysicsObject>().iter()
                                .filter(|(id, obj)| !obj.fixed)
                                .map(|(id, obj)| (id, obj.clone()))
                                .collect()
                        }),
                    ]
                ).unwrap());
            }

            if recv {
                for channel in NET_CHANNELS {
                    while let Some(ref data) = self.server.receive_message(*client, channel) {
                        match deserialize::<ClientMessages>(data) {
                            Ok(new_msgs) =>
                                for msg in new_msgs {
                                    msgs.push(msg);
                                }
                            Err(err) => 
                                eprintln!("{}", err)
                        }
                    }
                }
            }
        }

        self.transport.send_packets(&mut self.server);

        for msg in msgs {
            self.handle_msg(msg).await;
        }
    }
}
