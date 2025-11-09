use crate::*;
use std::f32::consts::PI;

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct Cube {
//     pub pos: Vec3,
//     pub size: Vec3,
//     pub rot: Vec3,
// }

fn conv_vec_1(vec: Vec3) -> Vector3<f32> {
    Vector3::new(vec.x, vec.y, vec.z)
}

pub fn conv_vec_2(vector: Vector3<f32>) -> Vec3 {
    vec3(vector.x, vector.y, vector.z)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obb {
    pub aabb: Aabb,
    pub iso: Isometry<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Obb(Obb),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsObject {
    pub shape: Shape,
    pub vel: Vec3,
    pub friction: f32,
    pub fixed: bool,
    pub on_ground: bool,
}

impl Obb {
    pub fn from_pos_size(pos: Vec3, size: Vec3) -> Self {
        Self {
            aabb: Aabb::from_half_extents(
                Point::new(0.0, 0.0, 0.0),
                Vector3::new(size.x / 2.0, size.y / 2.0, size.z / 2.0)
            ),
            iso: Isometry::new(
                conv_vec_1(pos),
                Vector3::new(PI, 0.0, 0.0)
            ),
        }
    }

    pub fn from_points(points: &[Point<f32>]) -> Self {
        let (iso, cuboid) = points_to_obb(points);
        Self {
            aabb: cuboid.local_aabb(),
            iso,
        }
    }
}

impl PhysicsObject {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            vel: vec3(0.0, 0.0, 0.0),
            friction: 1.02,
            fixed: false,
            on_ground: false,
        }
    }

    pub fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }

    pub fn vel(mut self, vel: Vec3) -> Self {
        self.vel = vel;
        self
    }

    // pub fn friction(mut self, friction: f32) -> Self {
    //     self.friction = friction;
    //     self
    // }

    pub fn pos(&self) -> Vec3 {
        conv_vec_2(match &self.shape {
            Shape::Obb(obb) => {
                obb.iso.translation.vector
            }
        })
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        match &mut self.shape {
            Shape::Obb(obb) => {
                obb.iso.translation.vector = conv_vec_1(pos);
            }
        }
    }

    pub fn move_pos(&mut self, delta_pos: Vec3) {
        self.set_pos(self.pos() + delta_pos);
    }

    pub fn yaw(&self) -> f32 {
        match &self.shape {
            Shape::Obb(obb) => {
                obb.iso.rotation.euler_angles().2
            }
        }
    }

    pub fn yaw_vec(&self) -> Vec3 {
        let v = Vec2::from_angle(self.yaw());
        vec3(v.x, 0.0, v.y)
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        match &mut self.shape {
            Shape::Obb(obb) => {
                let (roll, pitch, _) = obb.iso.rotation.euler_angles();
                obb.iso.rotation = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
            }
        }
    }

    pub fn intersects(&self, rcs: &Self) -> bool {
        match &self.shape {
            Shape::Obb(obb) => {
                match &rcs.shape {
                    Shape::Obb(obb2) => {
                        obb.aabb.transform_by(&obb.iso).intersection(
                            &obb2.aabb.transform_by(&obb2.iso)
                        ).is_some()
                    }
                }
            }
        }
    }

    pub fn standing_on(&self, rcs: &Self) -> (bool, f32) {
        // TODO rename to `standing_on_cube` or similar

        let half_y = match &self.shape {
            Shape::Obb(obb) => {
                obb.aabb.transform_by(&obb.iso).half_extents().y
            }
        };

        let feet = self.pos().y - half_y;

        match &rcs.shape {
            Shape::Obb(obb) => {
                let ground = rcs.pos().y + obb.aabb.transform_by(&obb.iso).half_extents().y;
                (f32::abs(feet - ground) < 0.1, ground + half_y)
            }

            // NOTE return false for eventual non-cube shapes
        }
    }

    pub fn bounding_box(&self) -> Vec3 {
        match &self.shape {
            Shape::Obb(obb) => {
                conv_vec_2(obb.aabb.transform_by(&obb.iso).extents())
            }
        }
    }
}

// impl Cube {
//     pub fn new(pos: Vec3, size: Vec3) -> Self {
//         Self {
//             pos,
//             size,
//             rot: vec3(-1.0, 0.0, 0.0),
//         }
//     }

//     fn mins(&self) -> Vec3 {
//         vec3(self.pos.x - self.size.x * 0.5, self.pos.y - self.size.y * 0.5, self.pos.z - self.size.z * 0.5)
//     }

//     fn maxs(&self) -> Vec3 {
//         vec3(self.pos.x + self.size.x * 0.5, self.pos.y + self.size.y * 0.5, self.pos.z + self.size.z * 0.5)
//     }

//     pub fn intersects(&self, rcs: &Self) -> bool {
//         let a_min = self.mins();
//         let a_max = self.maxs();
//         let b_min = rcs.mins();
//         let b_max = rcs.maxs();

//         !(a_max.x < b_min.x || a_min.x > b_max.x ||
//           a_max.y < b_min.y || a_min.y > b_max.y ||
//           a_max.z < b_min.z || a_min.z > b_max.z)
//     }

//     pub fn standing_on(&self, rcs: &Self) -> bool {
//         f32::abs((self.pos.y - self.size.y / 2.0) - (rcs.pos.y + rcs.size.y / 2.0)) < 0.5
//         && self.intersects(rcs)
//     }
// }
