#![allow(non_snake_case)]

use crate::*;

pub type ServerMessages = Vec<ServerMessage>;
pub type ClientMessages = Vec<ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    AssignId(Entity),
    Ecs {
        PhysicsObject: Vec<(Entity, PhysicsObject)>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    PosVel(Vec3, Vec3),
}
