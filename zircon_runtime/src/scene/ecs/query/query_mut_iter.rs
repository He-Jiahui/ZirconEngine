use std::marker::PhantomData;

use super::query_state::{CachedArchetypePlan, find_cached_archetype_plan};
use crate::scene::World;
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryFilter, QueryMutData, QueryState,
    StableEntityLocation,
};

/// Mutable full-query iterator over a call-local stable candidate snapshot.
pub struct QueryMutIter<'world, 'state, D, F = ()>
where
    D: QueryMutData,
    F: QueryFilter,
{
    world: *mut World,
    plans: &'state [CachedArchetypePlan],
    candidates: Vec<StableEntityLocation>,
    component_locations: Vec<ComponentStorageLocation>,
    index: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F> QueryMutIter<'world, 'state, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world mut World,
        plans: &'state [CachedArchetypePlan],
        ticks: ChangeTickWindow,
    ) -> Self {
        let candidates = world
            .stable_query_location_iter(plans.iter().map(CachedArchetypePlan::archetype_id))
            .collect();
        Self {
            world,
            plans,
            candidates,
            component_locations: Vec::new(),
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, 'state, D, F> Iterator for QueryMutIter<'world, 'state, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(stable_location) = self.candidates.get(self.index).copied() {
            self.index += 1;
            let plan =
                find_cached_archetype_plan(self.plans, stable_location.location.archetype_id)?;
            let world = unsafe { &*self.world };
            if !plan.write_component_locations(
                world,
                stable_location,
                &mut self.component_locations,
            ) {
                continue;
            }
            let entity = stable_location.stable_id;
            if F::matches_component_locations(world, entity, &self.component_locations, self.ticks)
            {
                // Candidate entities are unique and the query access descriptor
                // prevents overlapping mutable component access.
                return unsafe {
                    D::fetch_mut_with_component_locations(
                        &mut *self.world,
                        entity,
                        &self.component_locations,
                        self.ticks,
                    )
                };
            }
        }
        None
    }
}

impl<D, F> QueryState<D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn iter_mut<'world, 'state>(
        &'state mut self,
        world: &'world mut World,
    ) -> QueryMutIter<'world, 'state, D, F> {
        self.iter_mut_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub(crate) fn iter_mut_with_ticks<'world, 'state>(
        &'state mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> QueryMutIter<'world, 'state, D, F> {
        self.update_cache(world);
        QueryMutIter::new(world, self.cached_archetype_plans(), ticks)
    }
}
