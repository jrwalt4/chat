//! World datastructure to hold entities, components, and resources.

use std::collections::HashMap;

use crate::{
    component::{ArchetypeId, ArchetypeManager},
    entity::{EntityId, EntityManager},
};

#[derive(Default)]
pub struct World {
    entities: EntityManager,
    entity_index: HashMap<EntityId, ArchetypeId>,
    archetypes: ArchetypeManager,
}

impl World {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn archetypes(&self) -> &ArchetypeManager {
        &self.archetypes
    }
}
