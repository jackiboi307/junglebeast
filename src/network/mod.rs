mod message;
pub use message::*;

mod handler;
pub use handler::*;

#[macro_export]
macro_rules! clone_column {
    ($self:ident, $i:ident) => {
        $self.ecs.query::<&$i>()
            .iter().map(|(id, e)| (id, e.clone())).collect()
    }
}

pub use crate::clone_column;
