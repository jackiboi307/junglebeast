use crate::*;
use std::any::TypeId;
use serde::{Deserialize, Serialize};
use hecs::{*, serialize::column::*};

#[derive(Serialize, Deserialize)]
enum ComponentId {
    PhysicsObject,
}

pub struct Serializer;

impl SerializeContext for Serializer {
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

        try_serialize_id::<PhysicsObject, _, _>(archetype, &ComponentId::PhysicsObject, &mut out)?;
        out.end()
    }

    fn serialize_components<S: serde::ser::SerializeTuple>(
            &mut self,
            archetype: &Archetype,
            mut out: S,
        ) -> Result<S::Ok, S::Error> {

        try_serialize::<(u8, u8), _>(archetype, &mut out)?;
        out.end()
    }
}
