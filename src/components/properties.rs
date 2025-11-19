use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Properties {
    pub spawn: Option<bool>,
    pub launch: Option<Vec3>,
}
