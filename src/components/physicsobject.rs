use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cube {
    pub pos: Vec3,
    pub size: Vec3,
    pub rot: Vec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsObject {
    pub cube: Cube,
    pub vel: Vec3,
    pub friction: f32,
    pub fixed: bool,
    pub on_ground: bool,
}

impl Cube {
    pub fn new(pos: Vec3, size: Vec3) -> Self {
        Self {
            pos,
            size,
            rot: vec3(-1.0, 0.0, 0.0),
        }
    }

    fn mins(&self) -> Vec3 {
        vec3(self.pos.x - self.size.x * 0.5, self.pos.y - self.size.y * 0.5, self.pos.z - self.size.z * 0.5)
    }

    fn maxs(&self) -> Vec3 {
        vec3(self.pos.x + self.size.x * 0.5, self.pos.y + self.size.y * 0.5, self.pos.z + self.size.z * 0.5)
    }

    pub fn intersects(&self, rcs: &Self) -> bool {
        let a_min = self.mins();
        let a_max = self.maxs();
        let b_min = rcs.mins();
        let b_max = rcs.maxs();

        !(a_max.x < b_min.x || a_min.x > b_max.x ||
          a_max.y < b_min.y || a_min.y > b_max.y ||
          a_max.z < b_min.z || a_min.z > b_max.z)
    }

    pub fn standing_on(&self, rcs: &Self) -> bool {
        f32::abs((self.pos.y - self.size.y / 2.0) - (rcs.pos.y + rcs.size.y / 2.0)) < 0.5
        && self.intersects(rcs)
    }
}

impl PhysicsObject {
    pub fn new(cube: Cube) -> Self {
        Self {
            cube,
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
}
