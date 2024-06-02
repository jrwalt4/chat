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

pub trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Item<'world, 'state>: SystemParam<State = Self::State>;

    unsafe fn get_item<'world, 'state>(
        state: &'state mut Self::State,
        world: &'world World,
    ) -> Self::Item<'world, 'state>;
}

pub type SystemParamItem<'world, 'state, P> = <P as SystemParam>::Item<'world, 'state>;

impl<P0: SystemParam, P1: SystemParam> SystemParam for (P0, P1) {
    type State = (P0::State, P1::State);
    type Item<'world, 'state> = (
        SystemParamItem<'world, 'state, P0>,
        SystemParamItem<'world, 'state, P1>,
    );
    unsafe fn get_item<'world, 'state>(
        state: &'state mut Self::State,
        world: &'world World,
    ) -> Self::Item<'world, 'state> {
        let (state0, state1) = state;
        (P0::get_item(state0, world), P1::get_item(state1, world))
    }
}

impl<P0: SystemParam, P1: SystemParam, P2: SystemParam> SystemParam for (P0, P1, P2) {
    type State = (P0::State, P1::State, P2::State);
    type Item<'world, 'state> = (
        SystemParamItem<'world, 'state, P0>,
        SystemParamItem<'world, 'state, P1>,
        SystemParamItem<'world, 'state, P2>,
    );
    unsafe fn get_item<'world, 'state>(
        state: &'state mut Self::State,
        world: &'world World,
    ) -> Self::Item<'world, 'state> {
        let (state0, state1, state2) = state;
        (
            P0::get_item(state0, world),
            P1::get_item(state1, world),
            P2::get_item(state2, world),
        )
    }
}

pub trait SystemFunction<Hint>: Send + Sync + 'static {
    type Input;
    type Output;
    type Params: SystemParam;

    fn run(&self, input: Self::Input, params: SystemParamItem<Self::Params>) -> Self::Output;
}

impl<In, Out, P0: SystemParam, F: Send + Sync + 'static> SystemFunction<fn(In, P0) -> Out> for F
where
    for<'a> &'a F: Fn(In, SystemParamItem<'_, '_, P0>) -> Out,
{
    type Input = In;
    type Output = Out;
    type Params = P0;

    fn run(&self, input: Self::Input, params: SystemParamItem<Self::Params>) -> Self::Output {
        fn call_inner<I, O, P>(f: impl Fn(I, P) -> O, i: I, p: P) -> O {
            f(i, p)
        }
        call_inner(self, input, params)
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
