use std::{
    collections::HashMap,
    hash::Hash,
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::component::ArchetypeId;

/// A handle for a specific entity.
///
/// Values 0 through 2^31-1 are reserved for model entities added
/// from input files, while values 2^31 through u32::MAX (2^32-1)
/// are for more temporary entities created and destroyed during
/// simulation (notifications, intermediate artifacts, etc.).
/// It is up to the user to ensure entity id's are unique for values
/// in the lower range, the upper range is autoincremented.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Entity(u32);

impl Entity {
    const EXTERNAL_MIN: u32 = 0;
    const EXTERNAL_MAX: u32 = 0x7fffffff;
    const INTERNAL_MIN: u32 = 0x80000000;
    const INTERNAL_MAX: u32 = u32::MAX - 1;

    const NULL: Entity = Entity(u32::MAX);

    pub fn id(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Location {
    archetype: ArchetypeId,
    index: usize,
}

pub struct EntityLoc {
    pub entity: Entity,
    location: Location,
}

impl EntityLoc {
    pub fn archetype(&self) -> ArchetypeId {
        self.location.archetype
    }
    pub fn index(&self) -> usize {
        self.location.index
    }
}

pub(crate) struct EntityManager {
    entities: HashMap<Entity, Location>,
    next: AtomicU32,
    free_list: Vec<Entity>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            next: AtomicU32::new(Entity::INTERNAL_MIN),
            free_list: Default::default(),
        }
    }
}

impl EntityManager {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_id(&mut self, id: u32) -> Result<Entity, EntityError> {
        if id > Entity::EXTERNAL_MAX {
            return Err(EntityError::OutOfBounds);
        }
        let e = Entity(id);
        if self.entities.contains_key(&e) {
            return Err(EntityError::DuplicateID(id));
        }
        Ok(e)
    }

    /// Create [`Entity`] without checking whether it exists, but
    /// still checks if it is within bounds.
    ///
    /// This is not `unsafe`, but can result in unsound logic.  
    pub fn with_id_unchecked(id: u32) -> Result<Entity, EntityError> {
        if id > Entity::EXTERNAL_MAX {
            return Err(EntityError::OutOfBounds);
        }
        Ok(Entity(id))
    }

    pub(crate) fn create(&mut self) -> Result<Entity, EntityError> {
        match self.free_list.pop() {
            Some(ent) => Ok(ent),
            None => self.alloc().map(|id| Entity(id.into())),
        }
    }

    pub(crate) fn destroy(&mut self, entity: Entity) -> Result<EntityLoc, EntityError> {
        match self.entities.remove(&entity) {
            Some(location) => {
                self.free_list.push(entity);
                Ok(EntityLoc { entity, location })
            }
            None => Err(EntityError::DoesNotExist(entity)),
        }
    }

    fn alloc(&self) -> Result<NonZeroU32, EntityError> {
        let e_id = self.next.fetch_add(1, Ordering::Relaxed);
        // check for wrapping
        e_id.try_into().map_err(|_| EntityError::Overflow)
    }

    pub(crate) fn get_loc(&self, id: Entity) -> Option<EntityLoc> {
        self.entities
            .get_key_value(&id)
            .map(|(&entity, &location)| EntityLoc { entity, location })
    }
}

#[derive(Debug)]
pub enum EntityError {
    DoesNotExist(Entity),
    DuplicateID(u32),
    OutOfBounds,
    Overflow,
}
