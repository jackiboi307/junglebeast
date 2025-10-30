use macroquad::prelude::*;
use hecs::{
    Entity,
};
use serde::{Deserialize, Serialize};
use std::cmp::max;

mod network;
mod utils;
mod components;

#[cfg(not(server))]
mod client;

#[cfg(server)]
mod server;

pub use network::*;
pub use utils::*;
pub use components::*;

pub struct Shared {
    ecs: hecs::World,
}

impl Shared {
    fn new() -> Self {
        Self {
            ecs: hecs::World::new(),
        }
    }

    async fn handle_physics(&mut self, dt: f32, do_jump: Option<Entity>) {
        let mut bind = self.ecs.query::<(&mut PhysicsObject,)>();
        let (mut phys_objs, ids): (Vec<_>, Vec<_>) =
            bind.iter().map(|(id, (e,))| (e, id)).unzip();
        let len = phys_objs.len();

        for i in 0..len {
            let obj = &mut phys_objs[i];
            obj.vel.y -= 10.0 * dt;

            for j in 0..len {
                if i == j { continue }
                let (a, b) = phys_objs.split_at_mut(max(i, j));
                let (obj1, obj2) =
                    if i < j {
                        (&mut a[i], &mut b[0])
                    } else {
                        (&mut b[0], &mut a[j])
                    };

                obj1.on_ground = obj1.cube.standing_on(&obj2.cube);
                let collide = obj1.cube.intersects(&obj2.cube);

                if obj1.on_ground {
                    let friction = obj2.friction;
                    obj1.vel.x /= friction;
                    obj1.vel.z /= friction;
                    obj1.vel.y = 0.0;

                    if let Some(id) = do_jump {
                        if ids[i] == id {
                            obj1.vel.y = 5.0;
                        }
                    }

                } else if collide {
                    obj1.vel = (obj1.cube.pos - obj2.cube.pos).normalize();
                }
            }

            let obj = phys_objs.get_mut(i).unwrap();
            if !obj.fixed {
                obj.cube.pos += obj.vel * dt;
            }
        }
    }

    fn ray_intersection(&self, origin: Vec3, dir: Vec3, ignore_ids: &[Entity]) -> Option<(Vec3, Entity)> {
        // mainly ai generated!

        let mut result: Option<(Vec3, Entity)> = None;

        for (id, (obj,)) in self.ecs.query::<(&PhysicsObject,)>().iter() {
            if ignore_ids.contains(&id) {
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
    let mut client = client::Client::create(args.addr);
    client.start().await;
}

#[cfg(server)]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let mut server = server::Server::create(args.addr);
    server.start().await;
}
