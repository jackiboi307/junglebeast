#![allow(non_snake_case)]

use crate::*;

pub type ServerMessages = Vec<ServerMessage>;
pub type ClientMessages = Vec<ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    AssignId(Entity),
    Ecs(Columns),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    SetMoveState(MoveState),
    SetRotation(Vec3),
    Shot(Entity),
}

type Column<T> = Vec<(Entity, T)>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Columns {
    pub PhysicsObject: Column<PhysicsObject>,
    pub Player:        Column<Player>,
}
