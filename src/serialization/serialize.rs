use crate::*;
use super::*;

#[derive(Default)]
struct Context;

pub async fn serialize_world(world: &World) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    let options = bincode::options();
    let mut serializer = bincode::Serializer::new(&mut buffer, options);
    hecs::serialize::column::serialize(
        world,
        &mut Context,
        &mut serializer,
    ).unwrap();
    buffer
}

impl SerializeContext for Context {
    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype.component_types()
            .filter(|&t|
                t == TypeId::of::<PhysicsObject>() ||
            false ).count()
    }

    fn serialize_component_ids<S: serde::ser::SerializeTuple>(
            &mut self,
            archetype: &Archetype,
            mut out: S,
        ) -> Result<S::Ok, S::Error> {

        try_serialize_id::<PhysicsObject, _, _>(archetype, &Id::PhysicsObject, &mut out)?;
        out.end()
    }

    fn serialize_components<S: serde::ser::SerializeTuple>(
            &mut self,
            archetype: &Archetype,
            mut out: S,
        ) -> Result<S::Ok, S::Error> {

        try_serialize::<PhysicsObject, _>(archetype, &mut out)?;
        out.end()
    }
}
