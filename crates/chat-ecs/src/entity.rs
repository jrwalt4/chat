use std::{
    collections::HashMap,
    hash::Hash,
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::component::ArchetypeId;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Entity {
    Null,
    Id {
        id: NonZeroU32,
        gen: u16,
        flags: u16,
    },
}

impl Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Id { id, .. } => state.write_u32((*id).into()),
            Self::Null => state.write_u32(0),
        }
    }
}

impl Entity {
    pub(crate) fn with_id<I: TryInto<NonZeroU32>>(i: I) -> Self {
        i.try_into().map_or(Entity::Null, |id| Entity::Id {
            id,
            gen: Default::default(),
            flags: Default::default(),
        })
    }

    pub fn id(self) -> u32 {
        match self {
            Self::Null => 0,
            Self::Id { id, .. } => id.into(),
        }
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

#[derive(Default)]
pub(crate) struct EntityManager {
    entities: HashMap<Entity, Location>,
    next: AtomicU32,
    free_list: Vec<Entity>,
}

impl EntityManager {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn create(&mut self) -> Entity {
        match self.free_list.pop() {
            Some(mut ent) => {
                match &mut ent {
                    Entity::Id { ref mut gen, .. } => *gen += 1,
                    _ => panic!("Null Entity in free list"),
                }
                ent
            }
            None => Entity::with_id(self.alloc()),
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

    fn alloc(&self) -> NonZeroU32 {
        let e_id = self.next.fetch_add(1, Ordering::Relaxed);
        // check for wrapping
        e_id.try_into().expect("Too many entities")
    }

    pub(crate) fn get_loc(&self, id: Entity) -> Option<EntityLoc> {
        self.entities
            .get_key_value(&id)
            .map(|(&entity, &location)| EntityLoc { entity, location })
    }
}

pub enum EntityError {
    DoesNotExist(Entity),
}
