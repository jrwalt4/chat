//! A [SystemParam] to access components

use crate::component::{ArchetypeId, Component};

use std::marker::PhantomData;

pub struct Query<'a, C> {
    archetypes: Vec<&'a ArchetypeId>,
    component: PhantomData<&'a C>,
}

impl<'a, C: Component> Query<'a, C> {
    pub(crate) fn new(archetype_iter: impl IntoIterator<Item = &'a ArchetypeId>) -> Self {
        Self {
            archetypes: Vec::from_iter(archetype_iter),
            component: PhantomData,
        }
    }
}

pub type QueryResult<'a, C> = Result<Query<'a, C>, QueryError>;

pub enum QueryError {
    Empty,
}
