mod serialize;
mod deserialize;

pub use serialize::serialize_world;
pub use deserialize::deserialize_world;

// https://github.com/Ralith/hecs/blob/master/examples/serialize_to_disk.rs

// imported by modules:
use std::any::TypeId;
use serde::{Deserialize, Serialize};
use hecs::{*, serialize::column::*};

#[derive(Serialize, Deserialize)]
enum Id {
    PhysicsObject,
}
