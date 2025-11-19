use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Properties {
    pub launch: Option<Vec3>,
}
