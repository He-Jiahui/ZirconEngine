use crate::scene::ecs::{
    ChangeTickWindow, QueryDataAccess, QueryFilter, SystemParam, SystemParamAccess,
    SystemParamError,
};
use crate::scene::World;

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
}
