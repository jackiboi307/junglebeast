use crate::*;

use std::time::Duration;

impl Game {
    fn create_map(&mut self) {
        // self.ecs.spawn((physobj(
        //     vec3(0.0, 20.0, 0.0),
        //     vec3(1.0, 1.0, 1.0)),));
        self.ecs.spawn((physobj(
            vec3(0.0, -1.0, 0.0),
            vec3(60.0, 2.0, 60.0)).fixed(),));
        self.ecs.spawn((physobj(
            vec3(0.0, 0.5, 5.0),
            vec3(5.0, 1.0, 1.0)).fixed(),));
        self.ecs.spawn((physobj(
            vec3(0.0, 2.0, -5.0),
            vec3(5.0, 4.0, 1.0)).fixed(),));
    }

    pub async fn start_server(&mut self, addr: String) {
        use std::time::{Duration, Instant};
        use tokio::time::sleep;

        self.create_map();
        self.net.set_server(addr);

        let mut last_frame = Instant::now();

        let mut last_network_tick = Instant::now();
        let mut first = true;

        loop {
            let delta = last_frame.elapsed();
            let now = Instant::now();
            last_frame = now;

            self.handle_physics(delta.as_secs_f32(), false).await;

            if first || last_network_tick.elapsed().as_millis() >= 100 {
                self.handle_network(delta).await;
                last_network_tick = now;
                first = false;
            }

            sleep(Duration::from_secs_f32(1.0 / 30.0)).await;
        }
    }

    async fn handle_messages(&mut self, msgs: ClientMessages) {
        for msg in msgs {
            match msg {
                ClientMessage::Shared(msg) => {
                    match msg {
                        SharedMessage::Ecs {
                            PhysicsObject,
                        } => {
                            for (id, obj) in PhysicsObject {
                                if self.ecs.entity(id).is_err() {
                                    self.ecs.spawn_at(id, ());
                                }

                                self.ecs.insert(id, (obj,)).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_network(&mut self, duration: Duration) {
        let (mut server, transport) = self.net.server();

        server.update(duration);
        transport.update(duration, &mut server).unwrap();

        let mut msgss: Vec<Vec<_>> = Vec::new();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("{} connected", client_id);

                    server.send_message(client_id, DefaultChannel::ReliableUnordered, {
                        let mut msgs = Vec::new();

                        msgs.push({
                            let player = self.ecs.spawn((physobj(
                                vec3(0.0, 1.0, 0.0),
                                vec3(1.0, 2.0, 1.0)),
                            ));

                            ServerMessage::AssignId(player)
                        });

                        let mut buffer: Vec<u8> = Vec::new();
                        let options = bincode::options();
                        let mut serializer = bincode::Serializer::new(&mut buffer, options);
                        msgs.serialize(&mut serializer).unwrap();

                        buffer
                    });
                },
                _ => {}
            }
        }

        for client in server.clients_id_iter().collect::<Vec<_>>().iter() {
            server.send_message(*client, DefaultChannel::Unreliable, {
                let mut msgs = Vec::new();

                msgs.push(ServerMessage::Shared(SharedMessage::Ecs {
                    PhysicsObject: clone_column!(self, PhysicsObject)
                }));

                let mut buffer: Vec<u8> = Vec::new();
                let options = bincode::options();
                let mut serializer = bincode::Serializer::new(&mut buffer, options);
                msgs.serialize(&mut serializer).unwrap();

                buffer
            });

            while let Some(data) = server.receive_message(*client, DefaultChannel::Unreliable) {
                let data = data.to_vec();
                let options = bincode::options();
                let mut deserializer = bincode::Deserializer::from_slice(&data, options);
                msgss.push(ClientMessages::deserialize(&mut deserializer).unwrap());
            }
        }

        transport.send_packets(&mut server);

        for msgs in msgss {
            self.handle_messages(msgs).await;
        }
    }
}
