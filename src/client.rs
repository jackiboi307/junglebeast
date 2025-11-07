mod shared;
mod network;
mod utils;
mod components;
use crate::shared::*;

use renet::{RenetClient, DefaultChannel};
use renet_netcode::NetcodeClientTransport;

use std::time::Duration;

struct Client {
    shared: Shared,
    client: RenetClient,
    transport: NetcodeClientTransport,
    player: Entity,

    test_mesh: Option<Mesh>,
}

impl Client {
    fn create(addr: String) -> Self {
        let (client, transport) = create_client(addr);

        Self {
            shared: Shared::new(),
            client,
            transport,
            player: Entity::DANGLING,
            
            test_mesh: None,
        }
    }

    async fn start(&mut self) {
        self.load_textures().await;

        let mut x = 0.0;
        let mut switch = false;
        let bounds = 8.0;

        let world_up = vec3(0.0, 1.0, 0.0);
        let mut yaw: f32 = 1.18;
        let mut pitch: f32 = 0.0;

        let mut last_mouse_position: Vec2 = mouse_position().into();

        let look_speed = 0.1;

        let mut grabbed = true;
        set_cursor_grab(grabbed);
        show_mouse(!grabbed);

        let mut dt_accumulator = 0.0;

        loop {
            let delta = get_frame_time();
            dt_accumulator += delta;

            if (grabbed && is_key_pressed(KeyCode::Escape))
                || (!grabbed && is_mouse_button_pressed(MouseButton::Left)) {

                grabbed = !grabbed;
                set_cursor_grab(grabbed);
                show_mouse(!grabbed);
            }

            let mouse_position: Vec2 = mouse_position().into();
            let mouse_delta = mouse_position - last_mouse_position;

            if grabbed {
                last_mouse_position = mouse_position;

                yaw += mouse_delta.x * delta * look_speed;
                pitch += mouse_delta.y * delta * -look_speed;

                pitch = if pitch > 1.5 { 1.5 } else { pitch };
                pitch = if pitch < -1.5 { -1.5 } else { pitch };
            }

            let front = vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();

            if let Ok(mut obj) = self.shared.ecs.get::<&mut PhysicsObject>(self.player) {
                obj.cube.rot = front.cross(world_up).normalize();
            }

            x += if switch { 0.04 } else { -0.04 };
            if x >= bounds || x <= -bounds {
                switch = !switch;
            }

            let mut messages = ClientMessages::new();

            if let Ok((obj, player)) = self.shared.ecs.query_one_mut::<(&mut PhysicsObject, &mut Player)>(self.player) {
                player.moves.reset();

                messages.push(ClientMessage::SetRotation(obj.cube.rot));

                if is_key_down(KeyCode::W) { player.moves.forward = true; }
                if is_key_down(KeyCode::S) { player.moves.back = true; }
                if is_key_down(KeyCode::A) { player.moves.left = true; }
                if is_key_down(KeyCode::D) { player.moves.right = true; }

                if is_key_pressed(KeyCode::Space) {
                    player.moves.set_jump();
                }

                messages.push(ClientMessage::SetMoveState(player.moves.clone()));
            }

            if self.shared.ecs.entity(self.player).is_ok() {
                while dt_accumulator >= PHYSICS_STEP {
                    self.shared.handle_physics(PHYSICS_STEP).await;
                    dt_accumulator -= PHYSICS_STEP;
                }

                let (pos, up) = {
                    let obj = self.shared.ecs.get::<&PhysicsObject>(self.player).unwrap();
                    (obj.cube.pos, obj.clone().cube.rot.cross(front).normalize())
                };

                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((_, id)) = self.shared.ray_intersection(pos, front, &[self.player]) {
                        if self.shared.ecs.get::<&Player>(id).is_ok() {
                            messages.push(ClientMessage::Shot(id));
                        }
                    }
                }

                // if is_mouse_button_pressed(MouseButton::Left) {
                //     self.ecs.spawn((physobj(
                //         player_pos + front * 1.0,
                //         vec3(0.1, 0.1, 0.1)).vel(front * 10.0),));
                // }

                set_camera(&Camera3D {
                    position: pos,
                    up,
                    target: pos + front,
                    fovy: 2.05,
                    ..Default::default()
                });

                self.render().await;

            } else {
                draw_text("CONNECTING...", 10.0, 30.0, 30.0, WHITE);
            }

            self.handle_network(Duration::from_secs_f32(delta), messages).await;

            next_frame().await
        }
    }

    async fn load_textures(&mut self) {
        // TODO
        // see if there is a less complicated way that does not use unsafe,
        // to enable texture repeating

        // let backend = unsafe { get_internal_gl().quad_context };
        // let mut new_texture = async |filename| {
        //     let image = load_image(filename).await.expect("error loading texture");
        //     let id = backend.new_texture_from_rgba8(image.width, image.height, &image.bytes.into_boxed_slice());
        //     backend.texture_set_wrap(id, TextureWrap::Repeat, TextureWrap::Repeat);
        //     Texture2D::from_miniquad_texture(id)
        // };

        // self.shared.textures.insert("rust", new_texture("textures/rust.png").await);

        // self.textures.insert("rust", Texture2D::from_file_with_format(
        //     include_bytes!("../textures/rust.png"), None));
    }

    async fn handle_msg(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Ecs(columns) => {
                for (id, new_obj) in columns.PhysicsObject {
                    if self.shared.ecs.entity(id).is_err() {
                        self.shared.ecs.spawn_at(id, (new_obj,));
                    } else {
                        if let Ok(obj) = self.shared.ecs.query_one_mut::<&mut PhysicsObject>(id) {
                            let old_pos = obj.cube.pos;
                            let dist = obj.cube.pos.distance(new_obj.cube.pos);
                            // println!("{dist}");
                            *obj = new_obj;
                            if dist < 0.5 {
                                obj.cube.pos = old_pos.move_towards(obj.cube.pos, 0.005);
                            }
                        }
                    }
                }

                for (id, obj) in columns.Player {
                    self.shared.ecs.insert(id, (obj,)).unwrap();
                }

                for (_, wrapper) in columns.MeshWrapper {
                    self.test_mesh = Some(wrapper.to_mesh());
                }
            },
            ServerMessage::AssignId(id) => {
                self.player = id;
            }
        }
    }

    async fn handle_network(&mut self, duration: Duration, send_msgs: ClientMessages) {
        self.client.update(duration);
        self.transport.update(duration, &mut self.client).unwrap();

        let mut msgs = Vec::new();

        if self.client.is_connected() {
            self.client.send_message(DefaultChannel::ReliableOrdered,
                serialize(send_msgs).unwrap()
            );

            for channel in NET_CHANNELS {
                while let Some(ref data) = self.client.receive_message(channel) {
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

        self.transport.send_packets(&mut self.client).unwrap();

        for msg in msgs {
            self.handle_msg(msg).await;
        }
    }

    async fn render(&self) {
        clear_background(LIGHTGRAY);

        for (id, obj) in self.shared.ecs.query::<&PhysicsObject>().iter() {
            if id != self.player {
                draw_cube_wires(obj.cube.pos, obj.cube.size, BLACK);
            }
        }

        if let Some(mesh) = &self.test_mesh {
            draw_mesh(mesh);
        }
    
        set_default_camera();

        let center = (screen_width()/2.0, screen_height()/2.0);
        let crosshair_size = 12.0;
        draw_line(center.0 - crosshair_size, center.1, center.0 + crosshair_size, center.1, 1.0, BLACK);
        draw_line(center.0, center.1 - crosshair_size, center.0, center.1 + crosshair_size, 1.0, BLACK);

        draw_text("JUNGLEBEAST", 10.0, 30.0, 30.0, RED);

        if let Some((obj, player)) = self.shared.ecs.query_one::<(&PhysicsObject, &Player)>(self.player).unwrap().get() {
            let text = format!("fps: {}, hp: {} pos: {:.1} vel: {:.1}", get_fps(), player.hp(), obj.cube.pos, obj.vel);
            draw_text(&text, 10.0, 55.0, 30.0, GRAY);
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("JUNGLEBEAST"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let args = Args::parse();
    let mut client = Client::create(args.addr);
    client.start().await;
}
