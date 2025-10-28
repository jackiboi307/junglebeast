use macroquad::prelude::*;
use hecs::{
    Entity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod network;
mod serialization;

#[cfg(not(server))]
mod client;

#[cfg(server)]
mod server;

pub use network::*;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    net: NetworkHandler = NetworkHandler::new(),
} pub new }

impl Game {
    async fn handle_physics(&mut self, dt: f32, do_jump: bool) {
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
                    // TODO decide if this is retarded,
                    // or a viable client / server separation design
                    // some event handler system might be better
                    #[cfg(not(server))]
                    {
                        if *ids.get(i).unwrap() == self.player && do_jump {
                            obj.vel.y += 5.0;
                        }
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

#[cfg(not(server))]
fn conf() -> Conf {
    Conf {
        window_title: String::from("JUNGLEBEAST"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

use clap::{Parser, arg};

#[derive(Parser)]
struct Args {
    #[arg(help = "ip:port")]
    addr: String,
}

#[cfg(not(server))]
#[macroquad::main(conf)]
async fn main() {
    let args = Args::parse();
    let mut game = Game::new();
    game.start_client(args.addr).await;
}

#[cfg(server)]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let mut game = Game::new();
    game.start_server(args.addr).await;
}
