use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
enum SpawnKind {
    Player,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spawn {
    pub kind: SpawnKind,
    pub pos: Vec3,
}

impl Spawn {
    fn player(pos: Vec3) -> Self {
        Self {
            kind: SpawnKind::Player,
            pos,
        }
    }
}
