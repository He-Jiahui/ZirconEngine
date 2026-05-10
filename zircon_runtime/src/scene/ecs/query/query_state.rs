use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter,
    QueryIter, QueryMutData, SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::World;

#[derive(Clone, Debug)]
pub struct QueryState<D, F = ()> {
    access: QueryAccess,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<D, F> QueryState<D, F>
where
    D: QueryDataAccess,
    F: QueryFilter,
{
    pub fn new(world: &mut World) -> Self {
        Self::try_new(world).expect("query data must not request conflicting component access")
    }

    pub fn try_new(world: &mut World) -> Result<Self, QueryAccessError> {
        let mut access = QueryAccess::default();
        D::update_access(world, &mut access)?;
        F::update_access(world, &mut access)?;
        Ok(Self {
            access,
            _marker: PhantomData,
        })
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn conflicts_with<OtherD, OtherF>(&self, other: &QueryState<OtherD, OtherF>) -> bool {
        self.access.conflicts_with(&other.access)
    }
}

impl<D, F> QueryState<D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub fn iter<'world>(&self, world: &'world World) -> QueryIter<'world, D, F> {
        QueryIter::new(
            world,
            world.entity_ids_for_query(),
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }
}

impl<D, F> QueryState<D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn for_each_mut(&mut self, world: &mut World, mut f: impl FnMut(D::Item<'_>)) {
        let entities = world.entity_ids_for_query().to_vec();
        let ticks = ChangeTickWindow::all(world.read_change_tick());
        for entity in entities {
            if F::matches(world, entity, ticks) && D::matches_data(world, entity) {
                if let Some(item) = D::fetch_mut_with_ticks(world, entity, ticks) {
                    f(item);
                }
            }
        }
    }
}

impl<D, F> SystemParam for QueryState<D, F>
where
    D: QueryDataAccess,
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
        _state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        crate::scene::ecs::Query::new(world, ticks)
    }
}
