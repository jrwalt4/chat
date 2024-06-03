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

    fn init_state(world: &World) -> Self::State;

    unsafe fn from_state<'world, 'state>(
        state: &'state mut Self::State,
        world: &'world World,
    ) -> Self;
}

// pub type SystemParamItem<'world, 'state, P> = <P as SystemParam>::Item<'world, 'state>;

impl<P0: SystemParam, P1: SystemParam> SystemParam for (P0, P1) {
    type State = (P0::State, P1::State);

    fn init_state(world: &World) -> Self::State {
        (P0::init_state(world), P1::init_state(world))
    }
    unsafe fn from_state<'world, 'state>(
        state: &'state mut Self::State,
        world: &'world World,
    ) -> Self {
        let (state0, state1) = state;
        (P0::from_state(state0, world), P1::from_state(state1, world))
    }
}

impl<P0: SystemParam, P1: SystemParam, P2: SystemParam> SystemParam for (P0, P1, P2) {
    type State = (P0::State, P1::State, P2::State);

    fn init_state(world: &World) -> Self::State {
        (P0::init_state(world), P1::init_state(world), P2::init_state(world))
    }

    unsafe fn from_state<'world, 'state>(
        state: &'state mut Self::State,
        world: &'world World,
    ) -> Self {
        let (state0, state1, state2) = state;
        (
            P0::from_state(state0, world),
            P1::from_state(state1, world),
            P2::from_state(state2, world),
        )
    }
}

pub trait SystemParamFunction<Hint>: Send + Sync + 'static {
    type Input;
    type Output;
    type Params: SystemParam;

    fn run(&self, input: Self::Input, params: Self::Params) -> Self::Output;
}

impl<Out, P0: SystemParam, F: Send + Sync + 'static> SystemParamFunction<fn(P0) -> Out>
    for F
where
    for<'a> &'a F: Fn(P0) -> Out,
{
    type Input = ();
    type Output = Out;
    type Params = P0;

    fn run(&self, _input: Self::Input, params: Self::Params) -> Self::Output {
        fn call_inner<O, P>(f: impl Fn(P) -> O, p: P) -> O {
            f(p)
        }
        call_inner(self, params)
    }
}

impl<In, Out, P0: SystemParam, F: Send + Sync + 'static> SystemParamFunction<fn(In, P0) -> Out>
    for F
where
    for<'a> &'a F: Fn(In, P0) -> Out,
{
    type Input = In;
    type Output = Out;
    type Params = P0;

    fn run(&self, input: Self::Input, params: Self::Params) -> Self::Output {
        fn call_inner<I, O, P>(f: impl Fn(I, P) -> O, i: I, p: P) -> O {
            f(i, p)
        }
        call_inner(self, input, params)
    }
}

pub struct FunctionSystemDef<F, Hint>
where
    F: SystemParamFunction<Hint>,
{
    func: F,
    signature: PhantomData<fn() -> Hint>,
}

impl<F, Hint: 'static> FunctionSystemDef<F, Hint>
where
    F: SystemParamFunction<Hint>,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            signature: PhantomData
        }
    }

    pub fn into_system(self, world: &World) -> impl System<Input = F::Input, Output = F::Output> {
        FunctionSystem {
            func: self,
            param_state: <F::Params as SystemParam>::init_state(world)
        }
    }
}

pub struct FunctionSystem<F, Hint>
where
    F: SystemParamFunction<Hint>,
{
    func: FunctionSystemDef<F, Hint>,
    param_state: <F::Params as SystemParam>::State,
}

impl<F, Hint: 'static> System for FunctionSystem<F, Hint>
where
    F: SystemParamFunction<Hint>,
{
    type Input = <F as SystemParamFunction<Hint>>::Input;
    type Output = <F as SystemParamFunction<Hint>>::Output;

    fn run(&mut self, input: Self::Input, world: &World) -> Self::Output {
        let params = unsafe {
            <F::Params as SystemParam>::from_state(
                &mut self.param_state,
                world,
            )
        };
        self.func.func.run(input, params)
    }
}

#[test]
fn system_check() {

    struct DummyParam(i32);

    impl SystemParam for DummyParam {
        type State = i32;

        fn init_state(_world: &World) -> Self::State {
            2
        }

        unsafe fn from_state<'world, 'state>(
            state: &'state mut Self::State,
            _world: &'world World,
        ) -> Self {
            Self(*state)
        }
    }

    fn dummy_system(dummy: DummyParam) {
        assert_eq!(dummy.0, 2);
    }

    let world = World::new();
    
    let mut func_sys = FunctionSystemDef::new(dummy_system).into_system(&world);

    func_sys.run((), &world);
}
