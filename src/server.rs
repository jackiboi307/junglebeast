use crate::*;
use crate::serialization::*;

use serde::{Deserialize, Serialize};

use std::time::Duration;

impl Game {
    fn create_map(&mut self) {
        self.player = self.ecs.spawn((physobj(
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 2.0, 1.0)),));
        self.ecs.spawn((physobj(
            vec3(0.0, 20.0, 0.0),
            vec3(1.0, 1.0, 1.0)),));
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

        loop {
            let delta = last_frame.elapsed();
            last_frame = Instant::now();

            self.handle_physics(delta.as_secs_f32(), false).await;
            self.handle_network(delta).await;

            sleep(Duration::from_secs_f32(1.0 / 30.0)).await;
        }
    }

    async fn handle_network(&mut self, duration: Duration) {
        let (mut server, mut transport) = self.net.server();

        server.update(duration);
        transport.update(duration, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("{} connected", client_id);
                    server.send_message(client_id, DefaultChannel::ReliableOrdered, {
                        serialize_world(&self.ecs).await
                    });
                },
                _ => {}
            }
        }

        transport.send_packets(&mut server);
    }
}
