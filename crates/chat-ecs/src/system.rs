//! Systems that act on the world
//!

use std::marker::PhantomData;

use crate::world::World;

pub trait System: Send + Sync + 'static {
    type Input;
    type Output;

    fn run(&mut self, input: Self::Input, world: &World) -> Self::Output;
}

pub type SystemResult = Result<(), SystemError>;

pub enum SystemError {
    LogicError,
    MaxIterations,
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

pub trait SystemParamFunction<Hint>: Send + Sync + 'static {
    type Input;
    type Output;
    type Params: SystemParam;

    fn run(&self, input: Self::Input, params: SystemParamItem<Self::Params>) -> Self::Output;
}

impl<In, Out, P0: SystemParam, F: Send + Sync + 'static> SystemParamFunction<fn(In, P0) -> Out>
    for F
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

pub struct FunctionSystem<F, Hint>
where
    F: SystemParamFunction<Hint>,
{
    func: F,
    param_state: Option<<F::Params as SystemParam>::State>,
    signature: PhantomData<fn() -> Hint>,
}

impl<F, Hint: 'static> System for FunctionSystem<F, Hint>
where
    F: SystemParamFunction<Hint>,
{
    type Input = <F as SystemParamFunction<Hint>>::Input;
    type Output = <F as SystemParamFunction<Hint>>::Output;

    fn run(&mut self, input: Self::Input, world: &World) -> Self::Output {
        let params = unsafe {
            <F::Params as SystemParam>::get_item(
                self.param_state.as_mut().expect("No ParamState"),
                world,
            )
        };
        self.func.run(input, params)
    }
}
