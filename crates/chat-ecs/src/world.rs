//! World datastructure to hold entities, components, and resources.

use crate::{
    component::{ArchetypeManager, Component},
    entity::{Entity, EntityManager},
};

#[derive(Default)]
pub struct World {
    entities: EntityManager,
    archetypes: ArchetypeManager,
}

impl World {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn archetypes(&self) -> &ArchetypeManager {
        &self.archetypes
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        let loc = self.entities.get_loc(entity)?;
        let arch = self
            .archetypes
            .get_by_id(&loc.archetype())
            .expect("Archetype does not exist");
        let slice = unsafe {
            arch.get_column_as_slice::<C>()
                .expect("Column does not exist")
        };
        Some(&slice[loc.index()])
    }
}
