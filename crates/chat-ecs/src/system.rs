//! Systems that act on the world
//!
//! Systems are executed concurrently through async [`Future`]'s and request
//! access to components through a [`SystemContext`].

use std::marker::PhantomData;

use crate::{
    component::{ArchetypeId, Component},
    world::World,
};

pub trait System: Send + Sync + 'static {
    fn run(&self, context: &mut SystemContext<'_>) -> SystemResult;
}

/// Execution context for a system
pub struct SystemContext<'a> {
    world: &'a World,
}

pub type SystemResult = Result<(), SystemError>;

pub enum SystemError {
    LogicError,
    MaxIterations,
}

impl<'a> SystemContext<'a> {
    async fn query<C: Component>(&self) -> QueryResult<'a, C> {
        let query = Query::new(
            self.world
                .archetypes()
                .query_component::<C>()
                .ok_or(QueryError::Empty)?,
        );
        Ok(query)
    }
}

pub struct Query<'a, C> {
    archetypes: Vec<&'a ArchetypeId>,
    component: PhantomData<&'a C>,
}

impl<'a, C: Component> Query<'a, C> {
    fn new(archetype_iter: impl IntoIterator<Item = &'a ArchetypeId>) -> Self {
        Self {
            archetypes: Vec::from_iter(archetype_iter.into_iter()),
            component: PhantomData,
        }
    }
}

pub type QueryResult<'a, C> = Result<Query<'a, C>, QueryError>;

pub enum QueryError {
    Empty,
}
