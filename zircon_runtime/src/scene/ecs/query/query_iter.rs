use std::{marker::PhantomData, ptr::NonNull};

use super::query_state::{CachedArchetypePlan, find_cached_archetype_plan};
use crate::scene::World;
use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter,
    QueryState,
};
use crate::scene::world::{StableQueryLocationIter, StableWorldEntityIter};

enum QueryIterSource<'world> {
    Entities(StableWorldEntityIter<'world>),
    Cached(StableQueryLocationIter<'world>),
}

pub struct QueryIter<'world, 'entities, D, F = ()>
where
    D: QueryData,
    F: QueryFilter,
{
    world: &'world World,
    source: QueryIterSource<'world>,
    plans: Option<&'entities [CachedArchetypePlan]>,
    component_locations: Vec<ComponentStorageLocation>,
    change_detection_stats: ChangeDetectionScanStats,
    state: Option<NonNull<QueryState<D, F>>>,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, 'entities, D, F> QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub(crate) fn new(world: &'world World, ticks: ChangeTickWindow) -> Self {
        Self {
            world,
            source: QueryIterSource::Entities(world.entity_ids_for_query()),
            plans: None,
            component_locations: Vec::new(),
            change_detection_stats: ChangeDetectionScanStats::default(),
            state: None,
            ticks,
            _marker: PhantomData,
        }
    }

    pub(crate) fn new_cached_plans(
        world: &'world World,
        plans: &'entities [CachedArchetypePlan],
        ticks: ChangeTickWindow,
        state: &'entities QueryState<D, F>,
    ) -> Self {
        Self {
            world,
            source: QueryIterSource::Cached(
                world.stable_query_location_iter(
                    plans.iter().map(CachedArchetypePlan::archetype_id),
                ),
            ),
            plans: Some(plans),
            component_locations: Vec::new(),
            change_detection_stats: ChangeDetectionScanStats::default(),
            state: Some(NonNull::from(state)),
            ticks,
            _marker: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn uses_compiled_archetype_plans(&self) -> bool {
        matches!(self.source, QueryIterSource::Cached(_)) && self.plans.is_some()
    }
}

impl<'world, 'entities, D, F> Iterator for QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.source {
                QueryIterSource::Entities(entities) => {
                    let entity = entities.next()?;
                    if F::matches(self.world, entity, self.ticks)
                        && D::matches_data(self.world, entity)
                    {
                        if let Some(item) = D::fetch_with_ticks(self.world, entity, self.ticks) {
                            return Some(item);
                        }
                    }
                }
                QueryIterSource::Cached(locations) => {
                    let stable_location = locations.next()?;
                    let plans = self.plans.expect("cached query iterator must borrow plans");
                    let plan =
                        find_cached_archetype_plan(plans, stable_location.location.archetype_id)?;
                    if !plan.write_component_locations(
                        self.world,
                        stable_location,
                        &mut self.component_locations,
                    ) {
                        continue;
                    }
                    let entity = stable_location.stable_id;
                    if F::matches_component_locations_with_stats(
                        self.world,
                        entity,
                        &self.component_locations,
                        self.ticks,
                        &mut self.change_detection_stats,
                    ) {
                        if let Some(item) = D::fetch_with_component_locations(
                            self.world,
                            entity,
                            stable_location,
                            &self.component_locations,
                            self.ticks,
                        ) {
                            return Some(item);
                        }
                    }
                }
            }
        }
    }
}

impl<'world, 'entities, D, F> Drop for QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    fn drop(&mut self) {
        if let Some(state) = self.state {
            // SAFETY: the pointer originates from the QueryState borrow that also
            // owns the plans held for the iterator's complete lifetime.
            let state = unsafe { state.as_ref() };
            state.record_change_detection_stats(self.change_detection_stats);
        }
    }
}
