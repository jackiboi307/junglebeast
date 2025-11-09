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
    SetYaw(f32),
    Shot(Entity),
}

type Column<T> = Vec<(Entity, T)>;

#[allow(non_snake_case)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Columns {
    pub PhysicsObject: Column<PhysicsObject>,
    pub MeshWrapper:   Column<MeshWrapper>,
    pub Player:        Column<Player>,
}

impl Columns {
    pub fn ids(&self) -> Vec<&Entity> {
        let mut ids = Vec::new();
        for (id, _) in &self.PhysicsObject { if !ids.contains(&id) { ids.push(id) } };
        for (id, _) in &self.MeshWrapper   { if !ids.contains(&id) { ids.push(id) } };
        for (id, _) in &self.Player        { if !ids.contains(&id) { ids.push(id) } };
        ids
    }
}
