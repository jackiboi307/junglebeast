#![allow(unused_imports)]

pub use macroquad::{
    prelude::*,
    rand::*,
};
pub use hecs::{
    Entity,
    EntityBuilder,
    DynamicBundle,
};
pub use rapier3d::{
    prelude::*,
    parry::{
        bounding_volume::Aabb,
        math::{Isometry, Point},
        utils::obb as points_to_obb,
        na::{Vector3, UnitQuaternion, Quaternion, Matrix4, Vector4},
        query::{Ray, RayCast},
    },
};
pub use serde::{Deserialize, Serialize};

pub use crate::network::*;
pub use crate::utils::*;
pub use crate::components::*;
pub use crate::physics::*;

use gltf::{
    image::Source,
    scene::Transform,
};
use serde_json::{
    from_value,
    from_str,
};

pub const PHYSICS_STEP: f32 = 1.0 / 60.0;
pub const TEST_MAP: &'static str = "maps/test.glb";

// convert vectors between Vec3 (glam, macroquad) and Vector3 (nalgebra, rapier)
// TODO create custom conversion trait such as .conv()?

pub fn conv_vec_1(vec: Vec3) -> Vector3<f32> {
    Vector3::new(vec.x, vec.y, vec.z)
}

pub fn conv_vec_2(vector: Vector3<f32>) -> Vec3 {
    vec3(vector.x, vector.y, vector.z)
}

// stupid trait used once

trait AssumeOne: Iterator + Sized {
    fn assume_one(self) -> Self::Item;
}

impl<I> AssumeOne for I
where
    I: Iterator,
{
    fn assume_one(mut self) -> Self::Item {
        match self.next() {
            None => panic!("assume_one failed: iterator is empty"),
            Some(first) => {
                if self.next().is_some() {
                    eprintln!("assume_one warning: iterator had more than one item (ignoring)");
                }
                first
            }
        }
    }
}

// the absolute core

pub struct Shared {
    pub ecs: hecs::World,
    pub physics: Physics,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            ecs: hecs::World::new(),
            physics: Physics::new(),
        }
    }

    pub async fn load_map(&mut self, path: String) {
        let (document, buffers, images) = gltf::import(path).unwrap();
        let scene = document.scenes().assume_one();

        for node in scene.nodes() {
            let mut builder = EntityBuilder::new();

            if let Some(mesh) = node.mesh() {
                builder.add_bundle(self.handle_mesh(&buffers, &images, &node, &mesh));
            } else {
                let (pos, _, _) = node.transform().decomposed();
                builder.add(PointObject(pos.into()));
            }

            if let Some(extras) = node.extras() {
                let props: Properties = from_value(from_str(extras.get()).unwrap()).unwrap();
                builder.add(props);
            }

            self.ecs.spawn(builder.build());
        }
    }

    fn handle_mesh(&mut self,
            buffers: &Vec<gltf::buffer::Data>,
            images: &Vec<gltf::image::Data>,
            node: &gltf::Node,
            mesh: &gltf::Mesh) -> impl DynamicBundle {

        let primitive = mesh.primitives().assume_one();
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let indices: Vec<_> = reader
            .read_indices()
            .map(|indices| indices.into_u32().collect()).unwrap();
        let mut vertices: Vec<_> = reader
            .read_positions()
            .unwrap()
            .map(|pos| {
                let tr = node.transform().matrix();
                let tr = Matrix4::from(tr);
                let p = tr * Vector4::new(pos[0], pos[1], pos[2], 1.0);
                Vertex::new(p.x / p.w, p.y / p.w, p.z / p.w, 0.0, 0.0, WHITE)
            })
            .collect();
        for (i, uv) in reader.read_tex_coords(0).unwrap().into_f32().enumerate() {
            vertices[i].uv = uv.into();
        }
        let mut texture = images[
                primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture().unwrap()
                .texture().source().index()
            ].clone();
        texture.pixels = texture.pixels
            .chunks_exact(3)
            .map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .flatten().collect();

        // create collider
        // the handler is not added to the ecs

        self.physics.state.collider_set.insert(
            ColliderBuilder::trimesh(
                vertices
                    .clone()
                    .iter()
                    .map(|v| Point::new(v.position.x, v.position.y, v.position.z))
                    .collect::<Vec<_>>(),
                indices
                    .chunks(3)
                    .map(|i| [i[0], i[1], i[2]])
                    .collect::<Vec<_>>()
            )
            .unwrap()
            .restitution(0.5)
            .build()
        );

        // added to the ecs:

        (
            MeshWrapper {
                vertices: vertices.iter().map(|v| VertexWrapper {
                    position: vec3(v.position.x, v.position.y, v.position.z),
                    color: [255, 255, 255, 255],
                    uv: vec2(v.uv.x, v.uv.y),
                    normal: vec4(v.normal.x, v.normal.y, v.normal.z, 1.0),
                }).collect(),
                indices: indices.iter().map(|i| *i as u16).collect(),
                texture: Some(ImageWrapper {
                    width: texture.width.try_into().unwrap(),
                    height: texture.height.try_into().unwrap(),
                    bytes: texture.pixels,
                }),
            },
        )
    }

    pub async fn handle_physics(&mut self, _dt: f32) {
        self.physics.step();
    }

    // pub async fn _handle_physics(&mut self, dt: f32) {
    //     let ids: Vec<Entity> = self.ecs.query::<(&PhysicsObject,)>().iter().map(|(id, _)| id).collect();
    //     let len = ids.len();

    //     for i in 0..len {
    //         if let Ok(obj) = self.ecs.query_one_mut::<&mut PhysicsObject>(ids[i]) {
    //             if !obj.fixed {
    //                 obj.vel.y -= 10.0 * dt;
    //             }
    //         }

    //         if let Ok((obj, player)) = self.ecs.query_one_mut::<(&mut PhysicsObject, &mut Player)>(ids[i]) {
    //             Self::handle_movement(dt, &mut player.moves, obj);
    //         }

    //         if let Ok(obj) = self.ecs.query_one_mut::<&mut PhysicsObject>(ids[i]) {
    //             if obj.fixed { continue }

    //             let pos = obj.pos();
    //             obj.move_pos(obj.vel * dt);
    //             let direction = (obj.pos() - pos).normalize();
    //             let distance = obj.pos().distance(pos);

    //             // let origin = conv_vec_1(obj.pos()).into();
    //             let mut origin = obj.pos();
    //             origin.y -= obj.half_extents().y;
    //             let origin = conv_vec_1(origin);
    //             let ray = Ray::new(origin.into(), conv_vec_1(direction).into());

    //             for j in 0..len {
    //                 if i == j { continue }

    //                 let [obj, obj2] = self.ecs.query_many_mut::<&mut PhysicsObject, 2>([ids[i], ids[j]]);
    //                 let obj  = obj.unwrap();
    //                 let obj2 = obj2.unwrap();

    //                 let res = obj2.cast_ray(&ray, distance);

    //                 if let Some(distance) = res {
    //                     let vel = -direction * distance;
    //                     obj.vel = vel;
    //                     println!("{}", vel);
    //                 }
    //             }
    //         }

    //         // for j in 0..len {
    //         //     if i == j { continue }
    //         //     let [obj1, obj2] = self.ecs.query_many_mut::<&mut PhysicsObject, 2>([ids[i], ids[j]]);
    //         //     let obj1 = obj1.unwrap();
    //         //     let obj2 = obj2.unwrap();
    //         // }
    //     }
    // }

    // fn handle_movement(_dt: f32, state: &mut MoveState, obj: &mut PhysicsObject) {
    //     let move_speed = 0.1;
    //     let step_ws = obj.yaw_vec() * move_speed;
    //     let step_ad = vec3(-step_ws.z, 0.0, step_ws.x);

    //     if state.forward    { obj.vel += step_ws; }
    //     if state.back       { obj.vel -= step_ws; }
    //     if state.left       { obj.vel -= step_ad; }
    //     if state.right      { obj.vel += step_ad; }

    //     if state.get_jump() && obj.on_ground {
    //         obj.vel.y += 5.0;
    //     }
    // }
}

pub use clap::{Parser, arg};

#[derive(Parser)]
pub struct Args {
    #[arg(help = "ip:port")]
    pub addr: String,
}
