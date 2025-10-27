use crate::*;
use super::*;

#[derive(Default)]
struct Context {
    components: Vec<Id>,
}

pub async fn deserialize_world(data: &[u8])
        -> Result<hecs::World, Box<bincode::ErrorKind>> {

    let options = bincode::options();
    let mut deserializer = bincode::Deserializer::from_slice(data, options);
    hecs::serialize::column::deserialize(
        &mut Context::default(),
        &mut deserializer,
    )
}

impl DeserializeContext for Context {
    fn deserialize_component_ids<'de, A>(
        &mut self,
        mut seq: A,
    ) -> Result<ColumnBatchType, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        self.components.clear(); // Discard data from the previous archetype
        let mut batch = ColumnBatchType::new();
        while let Some(id) = seq.next_element()? {
            match id {
                Id::PhysicsObject => {
                    batch.add::<PhysicsObject>();
                }
            }
            self.components.push(id);
        }
        Ok(batch)
    }

    fn deserialize_components<'de, A>(
        &mut self,
        entity_count: u32,
        mut seq: A,
        batch: &mut ColumnBatchBuilder,
    ) -> Result<(), A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // Decode component data in the order that the component IDs appeared
        for component in &self.components {
            match *component {
                Id::PhysicsObject => {
                    deserialize_column::<PhysicsObject, _>(entity_count, &mut seq, batch)?;
                }
            }
        }
        Ok(())
    }
}
