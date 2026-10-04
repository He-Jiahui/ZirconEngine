use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError, WorkerCommandBuffer,
};
use crate::scene::World;

/// 允许组内冲突参数通过 p0、p1 等短借用依次使用；整组访问仍参与系统间的调度冲突检查。
pub struct ParamSet<P>
where
    P: ParamSetParam,
{
    _marker: PhantomData<fn() -> P>,
}

/// 保存本次调用的参数状态与变更窗口；取子参数时借用自身，前一子参数释放后才能取得下一项。
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State);

    fn retire_state(_world: &mut World, _state: &mut Self::State) {}

    /// Exposes the sole command lane admitted when this parameter set was initialized.
    fn deferred_command_buffer_mut(_state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        None
    }
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        P::record_performance_diagnostics(world, state);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        P::retire_state(world, state);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        P::deferred_command_buffer_mut(state)
    }
}

macro_rules! init_param_set_state {
    ($world:ident, $access:ident, $(($param:ident, $state:ident, $candidate:ident)),+ $(,)?) => {{
        // 每个子参数只与外层访问校验，允许组内依次借用同一数据；最终仍合并保守访问并集，并累计命令 lane 数。
        let outer_access = $access.clone();
        let mut deferred_command_lane_count = outer_access.deferred_command_lane_count();
        init_param_set_state!(
            @next $world,
            $access,
            outer_access,
            deferred_command_lane_count;
            ();
            $(($param, $state, $candidate)),+
        )
    }};
    (
        @next $world:ident,
        $access:ident,
        $outer_access:ident,
        $deferred_command_lane_count:ident;
        ($(($completed_param:ident, $completed_state:ident)),*);
        ($param:ident, $state:ident, $candidate:ident)
        $(, ($remaining_param:ident, $remaining_state:ident, $remaining_candidate:ident))* $(,)?
    ) => {{
        let mut $candidate = $outer_access.clone();
        let mut $state = match $param::init_state($world, &mut $candidate) {
            Ok(state) => state,
            Err(error) => {
                $($completed_param::retire_state($world, &mut $completed_state);)*
                return Err(error);
            }
        };
        let candidate_lanes = $candidate
            .deferred_command_lane_count()
            .saturating_sub($outer_access.deferred_command_lane_count());
        let Some(next_lane_count) = $deferred_command_lane_count.checked_add(candidate_lanes) else {
            $param::retire_state($world, &mut $state);
            $($completed_param::retire_state($world, &mut $completed_state);)*
            return Err(SystemParamError::MultipleDeferredCommandParams);
        };
        if next_lane_count > 1 {
            $param::retire_state($world, &mut $state);
            $($completed_param::retire_state($world, &mut $completed_state);)*
            return Err(SystemParamError::MultipleDeferredCommandParams);
        }
        $deferred_command_lane_count = next_lane_count;
        $access.merge_param_set_access(&$candidate);
        init_param_set_state!(
            @next $world,
            $access,
            $outer_access,
            $deferred_command_lane_count;
            ($(($completed_param, $completed_state),)* ($param, $state));
            $(($remaining_param, $remaining_state, $remaining_candidate)),*
        )
    }};
    (
        @next $world:ident,
        $access:ident,
        $outer_access:ident,
        $deferred_command_lane_count:ident;
        ($(($param:ident, $state:ident)),*);
    ) => {
        Ok(($($state,)*))
    };
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
        C::record_performance_diagnostics(world, &mut state.2);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
        C::retire_state(world, &mut state.2);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        if let Some(buffer) = C::deferred_command_buffer_mut(&mut state.2) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
        C::record_performance_diagnostics(world, &mut state.2);
        D::record_performance_diagnostics(world, &mut state.3);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
        C::retire_state(world, &mut state.2);
        D::retire_state(world, &mut state.3);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        if let Some(buffer) = C::deferred_command_buffer_mut(&mut state.2) {
            return Some(buffer);
        }
        if let Some(buffer) = D::deferred_command_buffer_mut(&mut state.3) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
        C::record_performance_diagnostics(world, &mut state.2);
        D::record_performance_diagnostics(world, &mut state.3);
        E::record_performance_diagnostics(world, &mut state.4);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
        C::retire_state(world, &mut state.2);
        D::retire_state(world, &mut state.3);
        E::retire_state(world, &mut state.4);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        if let Some(buffer) = C::deferred_command_buffer_mut(&mut state.2) {
            return Some(buffer);
        }
        if let Some(buffer) = D::deferred_command_buffer_mut(&mut state.3) {
            return Some(buffer);
        }
        if let Some(buffer) = E::deferred_command_buffer_mut(&mut state.4) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
        C::record_performance_diagnostics(world, &mut state.2);
        D::record_performance_diagnostics(world, &mut state.3);
        E::record_performance_diagnostics(world, &mut state.4);
        F::record_performance_diagnostics(world, &mut state.5);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
        C::retire_state(world, &mut state.2);
        D::retire_state(world, &mut state.3);
        E::retire_state(world, &mut state.4);
        F::retire_state(world, &mut state.5);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        if let Some(buffer) = C::deferred_command_buffer_mut(&mut state.2) {
            return Some(buffer);
        }
        if let Some(buffer) = D::deferred_command_buffer_mut(&mut state.3) {
            return Some(buffer);
        }
        if let Some(buffer) = E::deferred_command_buffer_mut(&mut state.4) {
            return Some(buffer);
        }
        if let Some(buffer) = F::deferred_command_buffer_mut(&mut state.5) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
        C::record_performance_diagnostics(world, &mut state.2);
        D::record_performance_diagnostics(world, &mut state.3);
        E::record_performance_diagnostics(world, &mut state.4);
        F::record_performance_diagnostics(world, &mut state.5);
        G::record_performance_diagnostics(world, &mut state.6);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
        C::retire_state(world, &mut state.2);
        D::retire_state(world, &mut state.3);
        E::retire_state(world, &mut state.4);
        F::retire_state(world, &mut state.5);
        G::retire_state(world, &mut state.6);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        if let Some(buffer) = C::deferred_command_buffer_mut(&mut state.2) {
            return Some(buffer);
        }
        if let Some(buffer) = D::deferred_command_buffer_mut(&mut state.3) {
            return Some(buffer);
        }
        if let Some(buffer) = E::deferred_command_buffer_mut(&mut state.4) {
            return Some(buffer);
        }
        if let Some(buffer) = F::deferred_command_buffer_mut(&mut state.5) {
            return Some(buffer);
        }
        if let Some(buffer) = G::deferred_command_buffer_mut(&mut state.6) {
            return Some(buffer);
        }
        None
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

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        A::record_performance_diagnostics(world, &mut state.0);
        B::record_performance_diagnostics(world, &mut state.1);
        C::record_performance_diagnostics(world, &mut state.2);
        D::record_performance_diagnostics(world, &mut state.3);
        E::record_performance_diagnostics(world, &mut state.4);
        F::record_performance_diagnostics(world, &mut state.5);
        G::record_performance_diagnostics(world, &mut state.6);
        H::record_performance_diagnostics(world, &mut state.7);
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        A::retire_state(world, &mut state.0);
        B::retire_state(world, &mut state.1);
        C::retire_state(world, &mut state.2);
        D::retire_state(world, &mut state.3);
        E::retire_state(world, &mut state.4);
        F::retire_state(world, &mut state.5);
        G::retire_state(world, &mut state.6);
        H::retire_state(world, &mut state.7);
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        if let Some(buffer) = A::deferred_command_buffer_mut(&mut state.0) {
            return Some(buffer);
        }
        if let Some(buffer) = B::deferred_command_buffer_mut(&mut state.1) {
            return Some(buffer);
        }
        if let Some(buffer) = C::deferred_command_buffer_mut(&mut state.2) {
            return Some(buffer);
        }
        if let Some(buffer) = D::deferred_command_buffer_mut(&mut state.3) {
            return Some(buffer);
        }
        if let Some(buffer) = E::deferred_command_buffer_mut(&mut state.4) {
            return Some(buffer);
        }
        if let Some(buffer) = F::deferred_command_buffer_mut(&mut state.5) {
            return Some(buffer);
        }
        if let Some(buffer) = G::deferred_command_buffer_mut(&mut state.6) {
            return Some(buffer);
        }
        if let Some(buffer) = H::deferred_command_buffer_mut(&mut state.7) {
            return Some(buffer);
        }
        None
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
