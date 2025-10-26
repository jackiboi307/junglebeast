use macroquad::prelude::*;
use crate::miniquad::{
    TextureWrap,
};
use hecs::{
    Entity,
};
use std::collections::HashMap;

macro_rules! gen_struct {
    (
        $svis:vis $sname:ident $( < $lt:lifetime > )?
        { $($fvis:vis $fname:ident : $t:ty = $e:expr),* $(,)? }
        $cvis:vis $cname:ident ) => {

        $svis struct $sname $( < $lt > )? {
            $(
                $fvis $fname: $t,
            )*
        }
        
        impl $( < $lt > )? $sname $( < $lt > )? {
            $cvis fn $cname() -> Self {
                Self {
                    $(
                        $fname: $e,
                    )*
                }
            }
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

struct Cube {
    pos: Vec3,
    size: Vec3,
    rot: Vec3,
}

impl Cube {
    fn new(pos: Vec3, size: Vec3) -> Self {
        Self {
            pos,
            size,
            rot: vec3(0.0, 0.0, 0.0),
        }
    }

    fn mins(&self) -> Vec3 {
        vec3(self.pos.x - self.size.x * 0.5, self.pos.y - self.size.y * 0.5, self.pos.z - self.size.z * 0.5)
    }

    fn maxs(&self) -> Vec3 {
        vec3(self.pos.x + self.size.x * 0.5, self.pos.y + self.size.y * 0.5, self.pos.z + self.size.z * 0.5)
    }

    fn intersects(&self, rcs: &Self) -> bool {
        let a_min = self.mins();
        let a_max = self.maxs();
        let b_min = rcs.mins();
        let b_max = rcs.maxs();

        !(a_max.x < b_min.x || a_min.x > b_max.x ||
          a_max.y < b_min.y || a_min.y > b_max.y ||
          a_max.z < b_min.z || a_min.z > b_max.z)
    }

    fn standing_on(&self, rcs: &Self) -> bool {
        f32::abs((self.pos.y - self.size.y / 2.0) - (rcs.pos.y + rcs.size.y / 2.0)) < 0.5
        && self.intersects(rcs)
    }
}

struct PhysicsObject {
    cube: Cube,
    vel: Vec3,
    friction: f32,
    fixed: bool,
}

fn physobj(pos: Vec3, size: Vec3) -> PhysicsObject {
    PhysicsObject::new(Cube::new(pos, size))
}

impl PhysicsObject {
    fn new(cube: Cube) -> Self {
        Self {
            cube,
            vel: vec3(0.0, 0.0, 0.0),
            friction: 1.02,
            fixed: false,
        }
    }

    fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }

    fn vel(mut self, vel: Vec3) -> Self {
        self.vel = vel;
        self
    }

    // fn friction(mut self, friction: f32) -> Self {
    //     self.friction = friction;
    //     self
    // }
}

gen_struct! { pub Game {
    ecs: hecs::World = hecs::World::new(),
    player: Entity = Entity::DANGLING,
    textures: HashMap<&'static str, Texture2D> = HashMap::new(),
} pub new }

impl Game {
    async fn init(&mut self) {
        self.load_textures().await;
        self.load_map();
    }

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

    fn load_map(&mut self) {
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

    async fn main(&mut self) {
        let mut x = 0.0;
        let mut switch = false;
        let bounds = 8.0;

        let world_up = vec3(0.0, 1.0, 0.0);
        let mut yaw: f32 = 1.18;
        let mut pitch: f32 = 0.0;

        let mut last_mouse_position: Vec2 = mouse_position().into();

        let move_speed = 0.1;
        let look_speed = 0.1;

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

            self.handle_physics(delta, do_jump).await;

            clear_background(LIGHTGRAY);

            let (player_pos, up) = {
                let cube = &self.ecs.get::<&PhysicsObject>(self.player).unwrap().cube;
                (cube.pos, cube.rot.cross(front).normalize())
            };

            set_camera(&Camera3D {
                position: player_pos,
                up,
                target: player_pos + front,
                fovy: 90.0,
                ..Default::default()
            });

            if is_mouse_button_pressed(MouseButton::Left) {
                self.ecs.spawn((physobj(
                    player_pos + front * 1.0,
                    vec3(0.1, 0.1, 0.1)).vel(front * 10.0),));
            }

            self.render().await;

            next_frame().await
        }
    }

    async fn handle_physics(&mut self, dt: f32, do_jump: bool) {
        // TODO
        // handle all player movement here, so that air control can be limited,
        // and friction doesn't impact walking

        let mut bind = self.ecs.query::<(&mut PhysicsObject,)>();
        let (mut phys_objs, ids): (Vec<_>, Vec<_>) =
            bind.iter().map(|(id, (e,))| (e, id)).unzip();
        let len = phys_objs.len();

        for i in 0..len {
            let obj = phys_objs.get_mut(i).unwrap();
            obj.vel.y -= 10.0 * dt;

            for j in 0..len {
                if i == j { continue }

                let collide = phys_objs.get(i).unwrap().cube
                    .intersects(&phys_objs.get(j).unwrap().cube);
                let standing_on = phys_objs.get(i).unwrap().cube
                    .standing_on(&phys_objs.get(j).unwrap().cube);

                if standing_on {
                    let friction = phys_objs.get(j).unwrap().friction;
                    let obj = phys_objs.get_mut(i).unwrap();
                    obj.vel.y = 0.0;
                    obj.vel.x /= friction;
                    obj.vel.z /= friction;

                    // jump
                    if *ids.get(i).unwrap() == self.player && do_jump {
                        obj.vel.y += 5.0;
                    }

                } else if collide {
                    let pos1 = phys_objs.get(i).unwrap().cube.pos;
                    let pos2 = phys_objs.get(j).unwrap().cube.pos;
                    let obj = phys_objs.get_mut(i).unwrap();
                    obj.vel = (pos1 - pos2).normalize();
                }
            }

            let obj = phys_objs.get_mut(i).unwrap();
            if !obj.fixed {
                obj.cube.pos += obj.vel * dt;
            }
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

    fn ray_intersection(&self, origin: Vec3, dir: Vec3, ignore_player: bool) -> Option<(Vec3, Entity)> {
        // mainly ai generated!

        let mut result: Option<(Vec3, Entity)> = None;

        for (id, (obj,)) in self.ecs.query::<(&PhysicsObject,)>().iter() {
            if id == self.player && ignore_player {
                continue
            }

            let cube = &obj.cube;
            let half = cube.size * 0.5;
            let min = cube.pos - half;
            let max = cube.pos + half;

            let mut tmin = f32::NEG_INFINITY;
            let mut tmax = f32::INFINITY;

            let mut check_axis = |o: f32, d: f32, a_min: f32, a_max: f32| -> bool {
                if d.abs() < 1e-8 {
                    return !(o < a_min || o > a_max);
                }
                let inv = 1.0 / d;
                let mut t0 = (a_min - o) * inv;
                let mut t1 = (a_max - o) * inv;
                if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
                if t0 > tmin { tmin = t0 }
                if t1 < tmax { tmax = t1 }
                tmin <= tmax
            };

            if !check_axis(origin.x, dir.x, min.x, max.x) ||
               !check_axis(origin.y, dir.y, min.y, max.y) ||
               !check_axis(origin.z, dir.z, min.z, max.z) {
                continue
            }

            if tmax < 0.0 { continue }
            let t_enter = tmin.max(0.0);
            
            if result.is_none() || t_enter < origin.distance(result.unwrap().0) {
                let res = origin + dir * t_enter;
                result = Some((res, id));
            }
        }

        return result;
    }
}

use clap::{Parser, arg};

#[derive(Parser)]
struct Args {
    #[arg()]
    ip: String,
    #[arg()]
    port: String,
}

#[cfg(not(server))]
#[macroquad::main(conf)]
async fn main() {
    let args = Args::parse();
    let mut game = Game::new();
    game.init().await;
    game.main().await;
}

#[cfg(server)]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let mut game = Game::new();
    let a = game.init().await;
    let b = game.main().await;
}
