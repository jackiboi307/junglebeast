#![allow(non_snake_case)]

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum SharedMessage {
    Ecs {
        PhysicsObject: Vec<(Entity, PhysicsObject)>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Shared(SharedMessage),
    AssignId(Entity),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Shared(SharedMessage),
}

pub type ServerMessages = Vec<ServerMessage>;
pub type ClientMessages = Vec<ClientMessage>;
