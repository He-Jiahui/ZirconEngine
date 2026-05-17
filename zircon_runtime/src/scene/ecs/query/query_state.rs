use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter,
    QueryIter, QueryMutData, SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::EntityId;
use crate::scene::World;

#[derive(Clone, Debug)]
pub struct QueryState<D, F = ()> {
    access: QueryAccess,
    cached_entities: Vec<EntityId>,
    cached_revision: u64,
    cache_rebuilds: u64,
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
        let mut state = Self {
            access,
            cached_entities: Vec::new(),
            cached_revision: u64::MAX,
            cache_rebuilds: 0,
            _marker: PhantomData,
        };
        state.update_cache(world);
        Ok(state)
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn conflicts_with<OtherD, OtherF>(&self, other: &QueryState<OtherD, OtherF>) -> bool {
        self.access.conflicts_with(&other.access)
    }

    pub fn update_cache(&mut self, world: &World) {
        let revision = world.query_cache_revision();
        if self.cached_revision == revision {
            return;
        }
        self.cached_entities.clear();
        let ticks = ChangeTickWindow::all(world.read_change_tick());
        self.cached_entities.extend(
            world
                .entity_ids_for_query()
                .iter()
                .copied()
                .filter(|entity| {
                    F::matches(world, *entity, ticks) && D::matches_data(world, *entity)
                }),
        );
        self.cached_revision = revision;
        self.cache_rebuilds = self.cache_rebuilds.saturating_add(1);
    }

    pub fn cached_entity_count(&self) -> usize {
        self.cached_entities.len()
    }

    pub fn cached_revision(&self) -> u64 {
        self.cached_revision
    }

    pub fn cache_rebuilds(&self) -> u64 {
        self.cache_rebuilds
    }
}

impl<D, F> QueryState<D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub fn iter<'world>(&self, world: &'world World) -> QueryIter<'world, 'world, D, F> {
        QueryIter::new(
            world,
            world.entity_ids_for_query(),
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_cached<'world, 'state>(
        &'state mut self,
        world: &'world World,
    ) -> QueryIter<'world, 'state, D, F> {
        self.update_cache(world);
        QueryIter::new(
            world,
            &self.cached_entities,
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
