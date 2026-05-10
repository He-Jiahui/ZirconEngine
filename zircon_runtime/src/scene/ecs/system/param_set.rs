use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError};
use crate::scene::World;

pub struct ParamSet<P>
where
    P: ParamSetParam,
{
    _marker: PhantomData<fn() -> P>,
}

pub struct ParamSetItem<'world, P>
where
    P: ParamSetParam,
{
    state: &'world mut P::State,
    world: *mut World,
    ticks: ChangeTickWindow,
}

pub trait ParamSetParam {
    type State: 'static;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError>;
}

impl<P> SystemParam for ParamSet<P>
where
    P: ParamSetParam,
{
    type State = P::State;
    type Item<'world> = ParamSetItem<'world, P>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        P::init_state(world, access)
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        ParamSetItem {
            state,
            world,
            ticks,
        }
    }
}

macro_rules! init_param_set_state {
    ($world:ident, $access:ident, $(($param:ident, $state:ident, $candidate:ident)),+ $(,)?) => {{
        let outer_access = $access.clone();
        $(
            let mut $candidate = outer_access.clone();
            let $state = {
                let state = $param::init_state($world, &mut $candidate)?;
                $access.merge_param_set_access(&$candidate);
                state
            };
        )+
        Ok(($($state,)+))
    }};
}

impl<A> ParamSetParam for (A,)
where
    A: SystemParam,
    A::State: 'static,
{
    type State = (A::State,);

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(world, access, (A, state_a, access_a))
    }
}

impl<A> ParamSetItem<'_, (A,)>
where
    A: SystemParam,
    A::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }
}

impl<A, B> ParamSetParam for (A, B)
where
    A: SystemParam,
    B: SystemParam,
    A::State: 'static,
    B::State: 'static,
{
    type State = (A::State, B::State);

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b)
        )
    }
}

impl<A, B> ParamSetItem<'_, (A, B)>
where
    A: SystemParam,
    B: SystemParam,
    A::State: 'static,
    B::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }
}

impl<A, B, C> ParamSetParam for (A, B, C)
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
{
    type State = (A::State, B::State, C::State);

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b),
            (C, state_c, access_c)
        )
    }
}

impl<A, B, C> ParamSetItem<'_, (A, B, C)>
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }

    pub fn p2(&mut self) -> C::Item<'_> {
        unsafe { C::get_param(self.world, &mut self.state.2, self.ticks) }
    }
}

impl<A, B, C, D> ParamSetParam for (A, B, C, D)
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
{
    type State = (A::State, B::State, C::State, D::State);

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b),
            (C, state_c, access_c),
            (D, state_d, access_d)
        )
    }
}

impl<A, B, C, D> ParamSetItem<'_, (A, B, C, D)>
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }

    pub fn p2(&mut self) -> C::Item<'_> {
        unsafe { C::get_param(self.world, &mut self.state.2, self.ticks) }
    }

    pub fn p3(&mut self) -> D::Item<'_> {
        unsafe { D::get_param(self.world, &mut self.state.3, self.ticks) }
    }
}

impl<A, B, C, D, E> ParamSetParam for (A, B, C, D, E)
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
{
    type State = (A::State, B::State, C::State, D::State, E::State);

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b),
            (C, state_c, access_c),
            (D, state_d, access_d),
            (E, state_e, access_e)
        )
    }
}

impl<A, B, C, D, E> ParamSetItem<'_, (A, B, C, D, E)>
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }

    pub fn p2(&mut self) -> C::Item<'_> {
        unsafe { C::get_param(self.world, &mut self.state.2, self.ticks) }
    }

    pub fn p3(&mut self) -> D::Item<'_> {
        unsafe { D::get_param(self.world, &mut self.state.3, self.ticks) }
    }

    pub fn p4(&mut self) -> E::Item<'_> {
        unsafe { E::get_param(self.world, &mut self.state.4, self.ticks) }
    }
}

impl<A, B, C, D, E, F> ParamSetParam for (A, B, C, D, E, F)
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    F: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
    F::State: 'static,
{
    type State = (A::State, B::State, C::State, D::State, E::State, F::State);

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b),
            (C, state_c, access_c),
            (D, state_d, access_d),
            (E, state_e, access_e),
            (F, state_f, access_f)
        )
    }
}

impl<A, B, C, D, E, F> ParamSetItem<'_, (A, B, C, D, E, F)>
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    F: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
    F::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }

    pub fn p2(&mut self) -> C::Item<'_> {
        unsafe { C::get_param(self.world, &mut self.state.2, self.ticks) }
    }

    pub fn p3(&mut self) -> D::Item<'_> {
        unsafe { D::get_param(self.world, &mut self.state.3, self.ticks) }
    }

    pub fn p4(&mut self) -> E::Item<'_> {
        unsafe { E::get_param(self.world, &mut self.state.4, self.ticks) }
    }

    pub fn p5(&mut self) -> F::Item<'_> {
        unsafe { F::get_param(self.world, &mut self.state.5, self.ticks) }
    }
}

impl<A, B, C, D, E, F, G> ParamSetParam for (A, B, C, D, E, F, G)
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    F: SystemParam,
    G: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
    F::State: 'static,
    G::State: 'static,
{
    type State = (
        A::State,
        B::State,
        C::State,
        D::State,
        E::State,
        F::State,
        G::State,
    );

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b),
            (C, state_c, access_c),
            (D, state_d, access_d),
            (E, state_e, access_e),
            (F, state_f, access_f),
            (G, state_g, access_g)
        )
    }
}

impl<A, B, C, D, E, F, G> ParamSetItem<'_, (A, B, C, D, E, F, G)>
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    F: SystemParam,
    G: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
    F::State: 'static,
    G::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }

    pub fn p2(&mut self) -> C::Item<'_> {
        unsafe { C::get_param(self.world, &mut self.state.2, self.ticks) }
    }

    pub fn p3(&mut self) -> D::Item<'_> {
        unsafe { D::get_param(self.world, &mut self.state.3, self.ticks) }
    }

    pub fn p4(&mut self) -> E::Item<'_> {
        unsafe { E::get_param(self.world, &mut self.state.4, self.ticks) }
    }

    pub fn p5(&mut self) -> F::Item<'_> {
        unsafe { F::get_param(self.world, &mut self.state.5, self.ticks) }
    }

    pub fn p6(&mut self) -> G::Item<'_> {
        unsafe { G::get_param(self.world, &mut self.state.6, self.ticks) }
    }
}

impl<A, B, C, D, E, F, G, H> ParamSetParam for (A, B, C, D, E, F, G, H)
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    F: SystemParam,
    G: SystemParam,
    H: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
    F::State: 'static,
    G::State: 'static,
    H::State: 'static,
{
    type State = (
        A::State,
        B::State,
        C::State,
        D::State,
        E::State,
        F::State,
        G::State,
        H::State,
    );

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        init_param_set_state!(
            world,
            access,
            (A, state_a, access_a),
            (B, state_b, access_b),
            (C, state_c, access_c),
            (D, state_d, access_d),
            (E, state_e, access_e),
            (F, state_f, access_f),
            (G, state_g, access_g),
            (H, state_h, access_h)
        )
    }
}

impl<A, B, C, D, E, F, G, H> ParamSetItem<'_, (A, B, C, D, E, F, G, H)>
where
    A: SystemParam,
    B: SystemParam,
    C: SystemParam,
    D: SystemParam,
    E: SystemParam,
    F: SystemParam,
    G: SystemParam,
    H: SystemParam,
    A::State: 'static,
    B::State: 'static,
    C::State: 'static,
    D::State: 'static,
    E::State: 'static,
    F::State: 'static,
    G::State: 'static,
    H::State: 'static,
{
    pub fn p0(&mut self) -> A::Item<'_> {
        unsafe { A::get_param(self.world, &mut self.state.0, self.ticks) }
    }

    pub fn p1(&mut self) -> B::Item<'_> {
        unsafe { B::get_param(self.world, &mut self.state.1, self.ticks) }
    }

    pub fn p2(&mut self) -> C::Item<'_> {
        unsafe { C::get_param(self.world, &mut self.state.2, self.ticks) }
    }

    pub fn p3(&mut self) -> D::Item<'_> {
        unsafe { D::get_param(self.world, &mut self.state.3, self.ticks) }
    }

    pub fn p4(&mut self) -> E::Item<'_> {
        unsafe { E::get_param(self.world, &mut self.state.4, self.ticks) }
    }

    pub fn p5(&mut self) -> F::Item<'_> {
        unsafe { F::get_param(self.world, &mut self.state.5, self.ticks) }
    }

    pub fn p6(&mut self) -> G::Item<'_> {
        unsafe { G::get_param(self.world, &mut self.state.6, self.ticks) }
    }

    pub fn p7(&mut self) -> H::Item<'_> {
        unsafe { H::get_param(self.world, &mut self.state.7, self.ticks) }
    }
}
