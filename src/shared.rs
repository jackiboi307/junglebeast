pub use macroquad::{
    prelude::*,
    rand::*,
};
pub use hecs::{
    Entity,
    EntityBuilder,
    DynamicBundle,
};
pub use parry3d::{
    bounding_volume::Aabb,
    math::{Isometry, Point},
    utils::obb as points_to_obb,
    na::{Vector3, UnitQuaternion, Quaternion, Matrix4, Vector4},
};
pub use serde::{Deserialize, Serialize};

pub use crate::network::*;
pub use crate::utils::*;
pub use crate::components::*;

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

pub struct Shared {
    pub ecs: hecs::World,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            ecs: hecs::World::new(),
        }
    }

    pub async fn load_map(&mut self, path: String) {
        let (document, buffers, images) = gltf::import(path).unwrap();
        let scene = document.scenes().assume_one();

        for node in scene.nodes() {
            let mut builder = EntityBuilder::new();

            if let Some(mesh) = node.mesh() {
                builder.add_bundle(self.handle_mesh(&buffers, &images, &node, mesh));
            } else {
                let (pos, _, _) = node.transform().decomposed();
                // println!("{:?}", pos);
                builder.add(PointObject(pos.into()));
            }

            if let Some(extras) = node.extras() {
                // println!("{:?}", extras);
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
            mesh: gltf::Mesh) -> impl DynamicBundle {

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

            PhysicsObject::new(Shape::Obb(Obb::from_points(
                &vertices.iter().map(|v| {
                    let p = v.position;
                    Point::new(p.x, p.y, p.z)
                }).collect::<Vec<Point<f32>>>()
            ))).fixed(),
        )
    }

    pub async fn handle_physics(&mut self, dt: f32) {
        let ids: Vec<Entity> = self.ecs.query::<(&PhysicsObject,)>().iter().map(|(id, _)| id).collect();
        let len = ids.len();

        for i in 0..len {
            {
                let mut obj = self.ecs.get::<&mut PhysicsObject>(ids[i]).unwrap();
                obj.vel.y -= 10.0 * dt;
                obj.on_ground = false;
            }

            for j in 0..len {
                if i == j { continue }
                let [obj1, obj2] = self.ecs.query_many_mut::<&mut PhysicsObject, 2>([ids[i], ids[j]]);
                let obj1 = obj1.unwrap();
                let obj2 = obj2.unwrap();

                let collide = obj1.intersects(&obj2);
                let (on_ground, ground) = obj1.standing_on(&obj2);
                let on_ground = collide && on_ground;

                obj1.on_ground = obj1.on_ground || on_ground;

                if on_ground {
                    let friction = obj2.friction;
                    obj1.vel.x /= friction;
                    obj1.vel.z /= friction;
                    obj1.vel.y = 0.0;

                    let pos = obj1.pos();
                    obj1.set_pos(vec3(pos.x, ground - 0.05, pos.z));

                } else if collide {
                    obj1.vel = (obj1.pos() - obj2.pos()).normalize();
                }
            }

            let obj = {
                if let Ok((obj, player)) = self.ecs.query_one_mut::<(&mut PhysicsObject, &mut Player)>(ids[i]) {
                    Self::handle_movement(dt, &mut player.moves, obj);
                    obj
                } else {
                    &mut self.ecs.get::<&mut PhysicsObject>(ids[i]).unwrap()
                }
            };

            let vel = obj.vel;
            if !obj.fixed {
                obj.move_pos(vel * dt);
            }
        }
    }

    fn handle_movement(dt: f32, state: &mut MoveState, obj: &mut PhysicsObject) {
        let move_speed = 0.1;
        let step_ws = obj.yaw_vec() * move_speed;
        let step_ad = vec3(-step_ws.z, 0.0, step_ws.x);

        if state.forward    { obj.vel += step_ws; }
        if state.back       { obj.vel -= step_ws; }
        if state.left       { obj.vel -= step_ad; }
        if state.right      { obj.vel += step_ad; }

        if state.get_jump() && obj.on_ground {
            obj.vel.y += 5.0;
        }
    }
}

pub use clap::{Parser, arg};

#[derive(Parser)]
pub struct Args {
    #[arg(help = "ip:port")]
    pub addr: String,
}
