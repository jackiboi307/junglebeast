use crate::*;

use renet::DefaultChannel;
use miniquad::{
    TextureWrap,
};

use std::time::{Duration, Instant};

impl Game {
    async fn load_textures(&mut self) {
        // TODO
        // see if there is a less complicated way that does not use unsafe,
        // to enable texture repeating

        let backend = unsafe { get_internal_gl().quad_context };
        let mut new_texture = async |filename| {
            let image = load_image(filename).await.expect("error loading texture");
            let id = backend.new_texture_from_rgba8(image.width, image.height, &image.bytes.into_boxed_slice());
            backend.texture_set_wrap(id, TextureWrap::Repeat, TextureWrap::Repeat);
            Texture2D::from_miniquad_texture(id)
        };

        self.textures.insert("rust", new_texture("textures/rust.png").await);

        // self.textures.insert("rust", Texture2D::from_file_with_format(
        //     include_bytes!("../textures/rust.png"), None));
    }

    pub async fn start_client(&mut self, addr: String) {
        self.load_textures().await;
        self.net.set_client(addr);

        let mut x = 0.0;
        let mut switch = false;
        let bounds = 8.0;

        let world_up = vec3(0.0, 1.0, 0.0);
        let mut yaw: f32 = 1.18;
        let mut pitch: f32 = 0.0;

        let mut last_mouse_position: Vec2 = mouse_position().into();

        let move_speed = 0.1;
        let look_speed = 0.1;

        let mut net_send = Interval::new(Duration::from_millis(200));
        let mut net_recv = Interval::new(Duration::from_millis(200));

        set_cursor_grab(true);
        show_mouse(false);

        loop {
            let delta = get_frame_time();

            let mouse_position: Vec2 = mouse_position().into();
            let mouse_delta = mouse_position - last_mouse_position;

            last_mouse_position = mouse_position;

            yaw += mouse_delta.x * delta * look_speed;
            pitch += mouse_delta.y * delta * -look_speed;

            pitch = if pitch > 1.5 { 1.5 } else { pitch };
            pitch = if pitch < -1.5 { -1.5 } else { pitch };

            let front = vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();

            if let Ok(mut obj) = self.ecs.get::<&mut PhysicsObject>(self.player) {
                obj.cube.rot = front.cross(world_up).normalize();
            }

            x += if switch { 0.04 } else { -0.04 };
            if x >= bounds || x <= -bounds {
                switch = !switch;
            }

            let do_jump = is_key_pressed(KeyCode::Space);

            if let Ok(mut obj) = self.ecs.get::<&mut PhysicsObject>(self.player) {
                let step_ws = vec3(obj.cube.rot.z, obj.cube.rot.y, -obj.cube.rot.x) * move_speed;
                let step_ad = obj.cube.rot * move_speed;

                if is_key_down(KeyCode::W) { obj.vel += step_ws; }
                if is_key_down(KeyCode::S) { obj.vel -= step_ws; }
                if is_key_down(KeyCode::A) { obj.vel -= step_ad; }
                if is_key_down(KeyCode::D) { obj.vel += step_ad; }
            }

            // if is_mouse_button_pressed(MouseButton::Left) {
            //     self.ecs.spawn((physobj(
            //         player_pos + front * 1.0,
            //         vec3(0.1, 0.1, 0.1)).vel(front * 10.0),));
            // }

            self.handle_physics(delta, do_jump).await;
            self.handle_network(Duration::from_secs_f32(delta), net_send.tick(), net_recv.tick()).await;

            if self.ecs.entity(self.player).is_err() { next_frame().await; continue }

            let (player_pos, up) = {
                let cube = &self.ecs.get::<&PhysicsObject>(self.player).unwrap().cube;
                (cube.pos, cube.rot.cross(front).normalize())
            };

            clear_background(LIGHTGRAY);

            set_camera(&Camera3D {
                position: player_pos,
                up,
                target: player_pos + front,
                fovy: 2.05,
                ..Default::default()
            });

            self.render().await;

            next_frame().await
        }
    }

    async fn handle_msg(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Shared(msg) => {
                match msg {
                    SharedMessage::Ecs {
                        PhysicsObject,
                    } => {
                        for (id, obj) in PhysicsObject {
                            if self.ecs.entity(id).is_err() {
                                self.ecs.spawn_at(id, (obj,));
                            } else {
                                if let Ok(new_obj) = self.ecs.query_one_mut::<&PhysicsObject>(id) {
                                    let dist = new_obj.cube.pos.distance(obj.cube.pos);
                                    if dist > 2.0 {
                                        self.ecs.insert(id, (obj,)).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            },
            ServerMessage::AssignId(id) => {
                self.player = id;
            }
        }
    }

    async fn handle_network(&mut self, duration: Duration, send: bool, recv: bool) {
        let (mut client, transport) = self.net.client();

        client.update(duration);
        transport.update(duration, &mut client).unwrap();

        let mut msgs: Vec<_> = Vec::new();

        if client.is_connected() {
            if recv {
                for channel in NET_CHANNELS {
                    while let Some(ref data) = client.receive_message(channel) {
                        match deserialize::<ServerMessages>(data) {
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

            if send {
                if self.ecs.entity(self.player).is_ok() {
                    client.send_message(DefaultChannel::Unreliable, serialize(
                        vec![
                            ClientMessage::Shared(SharedMessage::Ecs {
                                PhysicsObject: vec![(self.player,
                                    self.ecs.query_one::<&PhysicsObject>(self.player).unwrap().get().unwrap().clone())],
                            }),
                        ]
                    ).unwrap());
                }
            }
        }

        transport.send_packets(&mut client).unwrap();

        for msg in msgs {
            self.handle_msg(msg).await;
        }
    }

    async fn render(&self) {
        for (id, obj) in self.ecs.query::<&PhysicsObject>().iter() {
            if id != self.player {
                draw_cube_wires(obj.cube.pos, obj.cube.size, BLACK);
            }
        }
    
        set_default_camera();

        let center = (screen_width()/2.0, screen_height()/2.0);
        let crosshair_size = 12.0;
        draw_line(center.0 - crosshair_size, center.1, center.0 + crosshair_size, center.1, 1.0, BLACK);
        draw_line(center.0, center.1 - crosshair_size, center.0, center.1 + crosshair_size, 1.0, BLACK);

        draw_text("JUNGLEBEAST", 10.0, 30.0, 30.0, RED);
    }
}
