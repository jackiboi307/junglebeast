use macroquad::prelude::*;
use hecs::{
    Entity,
};

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
        f32::abs((self.pos.y - self.size.y / 2.0) - (rcs.pos.y + rcs.size.y / 2.0)) < 0.1
    }
}

struct PhysicsObject {
    cube: Cube,
    vel: Vec3,
    fixed: bool,
}

impl PhysicsObject {
    fn new(cube: Cube) -> Self {
        Self {
            cube,
            vel: vec3(0.0, 0.0, 0.0),
            fixed: false,
        }
    }

    fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
}

gen_struct! { pub Game {
    ecs: hecs::World = hecs::World::new(),
    player: Entity = Entity::DANGLING,
} pub new }

impl Game {
    fn init(&mut self) {
        self.player = self.ecs.spawn((PhysicsObject::new(Cube::new(
            vec3(2.0, 2.0, 2.0),
            vec3(1.0, 2.0, 1.0),
        )),));
        self.ecs.spawn((PhysicsObject::new(Cube::new(
            vec3(0.0, 20.0, 0.0),
            vec3(1.0, 1.0, 1.0),
        )),));
        self.ecs.spawn((PhysicsObject::new(Cube::new(
            vec3(0.0, 0.0, 0.0),
            vec3(20.0, 0.0, 20.0),
        )).fixed(),));
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

            self.render().await;

            next_frame().await
        }
    }

    async fn handle_physics(&mut self, dt: f32, do_jump: bool) {
        let mut bind = self.ecs.query::<(&mut PhysicsObject,)>();
        let (mut phys_objs, ids): (Vec<_>, Vec<_>) =
            bind.iter().map(|(id, (e,))| (e, id)).unzip();
        let len = phys_objs.len();

        for i in 0..len {
            let obj = phys_objs.get_mut(i).unwrap();
            obj.vel.y -= 10.0 * dt;

            let mut on_ground = false;

            for j in 0..len {
                if i == j { continue }

                let collide = phys_objs.get(i).unwrap().cube
                    .intersects(&phys_objs.get(j).unwrap().cube);
                let standing_on = phys_objs.get(i).unwrap().cube
                    .standing_on(&phys_objs.get(j).unwrap().cube);

                on_ground = on_ground || standing_on;

                if standing_on && collide {
                    let obj = phys_objs.get_mut(i).unwrap();
                    obj.vel.y = 0.0;

                } else if collide {
                    let vec = phys_objs.get(i).unwrap().cube.pos
                        .move_towards(phys_objs.get(j).unwrap().cube.pos, -0.1).normalize();
                    phys_objs.get_mut(i).unwrap().vel = vec;
                }
            }

            let obj = phys_objs.get_mut(i).unwrap();
            if *ids.get(i).unwrap() == self.player && do_jump && on_ground {
                obj.vel.y += 5.0;
            }
            if !obj.fixed {
                obj.cube.pos += obj.vel * dt;
            }
        }
    }

    async fn render(&self) {
        for (id, (obj,)) in self.ecs.query::<(&PhysicsObject,)>().iter() {
            if id != self.player {
                draw_cube(obj.cube.pos, obj.cube.size, None, BLUE);
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

#[macroquad::main(conf)]
async fn main() {
    let mut game = Game::new();
    game.init();
    game.main().await;
}
