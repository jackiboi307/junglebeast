use serde::{Serialize, Deserialize};

pub fn serialize(data: impl Serialize) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
    let mut buffer: Vec<u8> = Vec::new();
    let options = bincode::options();
    let mut serializer = bincode::Serializer::new(&mut buffer, options);
    data.serialize(&mut serializer)?;
    Ok(buffer)
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(data: &renet::Bytes) -> Result<T, Box<bincode::ErrorKind>> {
    let data = data.to_vec();
    let options = bincode::options();
    let mut deserializer = bincode::Deserializer::from_slice(&data, options);
    T::deserialize(&mut deserializer)
}

#[macro_export]
macro_rules! clone_column {
    ($self:ident, $type:ty) => {
        $self.shared.ecs.query::<$type>().iter()
            .map(|(id, obj)| (id, (*obj).clone()))
            .collect()
    }
}

// pub use crate::clone_column;
