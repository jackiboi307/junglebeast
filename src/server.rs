use crate::*;
use renet::{ServerEvent, DefaultChannel};
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

        let mut update   = Interval::new(Duration::from_millis(1000 / 30));
        let mut net_send = Interval::new(Duration::from_millis(200));
        let mut net_recv = Interval::new(Duration::from_millis(200));

        loop {
            if update.tick() {
                let delta = update.delta();

                self.handle_physics(delta.as_secs_f32(), false).await;
                self.handle_network(delta, net_send.tick(), net_recv.tick()).await;
            }
        }
    }

    async fn handle_msg(&mut self, msg: ClientMessage) {
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

    async fn handle_network(&mut self, duration: Duration, send: bool, recv: bool) {
        let (mut server, transport) = self.net.server();

        server.update(duration);
        transport.update(duration, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("{} connected", client_id);
                    server.send_message(client_id, DefaultChannel::ReliableUnordered, serialize(
                        vec![{
                            let player = self.ecs.spawn((physobj(
                                vec3(0.0, 1.0, 0.0),
                                vec3(1.0, 2.0, 1.0)),
                            ));

                            ServerMessage::AssignId(player)
                        }]
                    ).unwrap());
                },
                _ => {}
            }
        }

        let mut msgs: ClientMessages = Vec::new();

        for client in server.clients_id_iter().collect::<Vec<_>>().iter() {
            if send {
                server.send_message(*client, DefaultChannel::Unreliable, serialize(
                    vec![
                        ServerMessage::Shared(SharedMessage::Ecs {
                            PhysicsObject: clone_column!(self, PhysicsObject)
                        }),
                    ]
                ).unwrap());
            }

            if recv {
                for channel in NET_CHANNELS {
                    while let Some(ref data) = server.receive_message(*client, channel) {
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

        transport.send_packets(&mut server);

        for msg in msgs {
            self.handle_msg(msg).await;
        }
    }
}
