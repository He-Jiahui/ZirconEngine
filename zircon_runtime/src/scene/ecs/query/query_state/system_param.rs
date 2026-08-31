use crate::scene::World;
use crate::scene::ecs::{
    ChangeTickWindow, QueryDataAccess, QueryFilter, SystemParam, SystemParamAccess,
    SystemParamError,
};

use super::QueryState;

impl<D, F> SystemParam for QueryState<D, F>
where
    D: QueryDataAccess + 'static,
    F: QueryFilter,
{
    type State = QueryState<D, F>;
    type Item<'world> = crate::scene::ecs::Query<'world, D, F>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        let state = QueryState::<D, F>::try_new(world)?;
        access.add_query_access(state.access())?;
        Ok(state)
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        crate::scene::ecs::Query::new(world, state, ticks)
    }

    fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
        let query_stats = state.take_unreported_cache_stats();
        let change_detection_stats = state.take_unreported_change_detection_stats();
        world.record_ecs_query_cache_stats(query_stats);
        world.record_ecs_change_detection_stats(change_detection_stats);
    }
}
