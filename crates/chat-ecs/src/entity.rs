use std::{
    collections::HashMap,
    hash::Hash,
    sync::atomic::{AtomicU32, Ordering},
};

#[repr(transparent)]
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct EntityId(u32);

impl From<u32> for EntityId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<EntityId> for u32 {
    fn from(other: EntityId) -> u32 {
        other.0
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    id: EntityId,
    gen: u16,
    flags: u16,
}

impl Entity {
    pub(crate) fn with_id<I: Into<EntityId>>(i: I) -> Self {
        Entity {
            id: i.into(),
            gen: Default::default(),
            flags: Default::default(),
        }
    }

    pub fn id(&self) -> EntityId {
        self.id
    }
}

pub(crate) struct EntityManager {
    entities: HashMap<EntityId, Entity>,
    next: AtomicU32,
    free_list: Vec<Entity>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            next: AtomicU32::new(1),
            free_list: Vec::new(),
        }
    }
}

impl EntityManager {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn create(&mut self) -> Entity {
        match self.free_list.pop() {
            Some(mut ent) => {
                ent.gen += 1;
                ent
            }
            None => Entity::with_id(self.alloc()),
        }
    }

    pub(crate) fn destroy(&mut self, id: EntityId) -> Result<(), EntityError> {
        match self.entities.remove(&id) {
            Some(e) => {
                self.free_list.push(e);
                Ok(())
            }
            None => Err(EntityError::DoesNotExist(id)),
        }
    }

    fn alloc(&self) -> EntityId {
        let e_id = self.next.fetch_add(1, Ordering::Relaxed);
        // check for wrapping
        if e_id == 0 {
            panic!("Too many entities");
        }
        e_id.into()
    }
}

pub enum EntityError {
    DoesNotExist(EntityId),
}
