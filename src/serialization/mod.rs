// mod serialize;
// mod deserialize;

// pub use serialize::serialize;
// pub use deserialize::deserialize;

mod message;
pub use message::*;

// https://github.com/Ralith/hecs/blob/master/examples/serialize_to_disk.rs

// imported by modules:
use std::any::TypeId;
use serde::{Deserialize, Serialize};
use hecs::{*, serialize::column::*};

#[derive(Serialize, Deserialize)]
enum Id {
    PhysicsObject,
}

#[macro_export]
macro_rules! clone_column {
    ($self:ident, $i:ident) => {
        $self.ecs.query::<&$i>()
            .iter().map(|(id, e)| (id, e.clone())).collect()
    }
}

pub use crate::clone_column;
