use std::marker::PhantomData;

use super::query_state::{find_cached_archetype_plan, CachedArchetypePlan};
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryFilter, QueryMutData, QueryState,
    StableEntityLocation,
};
use crate::scene::World;

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
    pub(crate) unsafe fn new(
        world: *mut World,
        plans: &'state [CachedArchetypePlan],
        ticks: ChangeTickWindow,
    ) -> Self {
        unsafe {
            let candidates = World::query_stable_location_iter(
                world,
                plans.iter().map(CachedArchetypePlan::archetype_id),
            )
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
            let entity = stable_location.stable_id;
            let matches = {
                // SAFETY: projection/matching are candidate-local; the original
                // World loan freezes structure while earlier row items are live.
                let world = unsafe { &*self.world };
                plan.write_component_locations(
                    world,
                    stable_location,
                    &mut self.component_locations,
                ) && F::matches_component_locations(
                    world,
                    entity,
                    &self.component_locations,
                    self.ticks,
                )
            };
            if matches {
                // Candidates are unique; the unsafe data/filter contracts also
                // require candidate-local access and compatible retained items.
                return unsafe {
                    D::fetch_mut_with_component_locations(
                        self.world,
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
        let ticks = ChangeTickWindow::all(world.read_change_tick());
        // SAFETY: this genuine exclusive World loan bounds every yielded item.
        unsafe { self.iter_mut_with_ticks(world as *mut World, ticks) }
    }

    pub(crate) unsafe fn iter_mut_with_ticks<'world, 'state>(
        &'state mut self,
        world: *mut World,
        ticks: ChangeTickWindow,
    ) -> QueryMutIter<'world, 'state, D, F> {
        unsafe {
            self.update_cache(&*world);
            QueryMutIter::new(world, self.cached_archetype_plans(), ticks)
        }
    }
}
