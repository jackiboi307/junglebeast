use crate::*;

pub type ServerMessages = Vec<ServerMessage>;
pub type ClientMessages = Vec<ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    AssignId(Entity),
    Ecs(Columns),
    PhysicsState(RigidBodySet, ColliderSet),
    PhysicsDiff(PhysicsDiff),
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
    pub RigidBodyHandle: Column<RigidBodyHandle>,
    pub ColliderHandle:  Column<ColliderHandle>,
    pub MeshWrapper:     Column<MeshWrapper>,
    pub Player:          Column<Player>,
    pub PointObject:     Column<PointObject>,
    pub Properties:      Column<Properties>,
}

macro_rules! push {
    ($self:ident, $ids:ident, $t:tt) => {{
        for (id, _) in &$self.$t { if !$ids.contains(&id) { $ids.push(id) } };
    }}
}

impl Columns {
    pub fn ids(&self) -> Vec<&Entity> {
        let mut ids = Vec::new();
        push!(self, ids, RigidBodyHandle);
        push!(self, ids, ColliderHandle);
        push!(self, ids, MeshWrapper);
        push!(self, ids, Player);
        push!(self, ids, PointObject);
        push!(self, ids, Properties);
        ids
    }
}
