use std::{marker::PhantomData, ptr::NonNull};

use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter,
    QueryState, StableEntityLocation,
};
use crate::scene::{EntityId, World};

use super::cached_query_iter::cached_query_component_locations;

pub struct QueryIter<'world, 'entities, D, F = ()>
where
    D: QueryData,
    F: QueryFilter,
{
    world: &'world World,
    entities: &'entities [EntityId],
    locations: Option<&'entities [StableEntityLocation]>,
    component_locations: Option<&'entities [ComponentStorageLocation]>,
    component_location_offsets: Option<&'entities [usize]>,
    change_detection_stats: ChangeDetectionScanStats,
    // Cached slices keep the originating QueryState alive; the raw sink avoids
    // imposing a state-reference lifetime on read-only, non-cached iterators.
    state: Option<NonNull<QueryState<D, F>>>,
    index: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, 'entities, D, F> QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world World,
        entities: &'entities [EntityId],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities,
            locations: None,
            component_locations: None,
            component_location_offsets: None,
            change_detection_stats: ChangeDetectionScanStats::default(),
            state: None,
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    pub(crate) fn new_cached_locations(
        world: &'world World,
        entities: &'entities [EntityId],
        locations: &'entities [StableEntityLocation],
        component_locations: &'entities [ComponentStorageLocation],
        component_location_offsets: &'entities [usize],
        ticks: ChangeTickWindow,
        state: &'entities QueryState<D, F>,
    ) -> Self {
        Self {
            world,
            entities,
            locations: Some(locations),
            component_locations: Some(component_locations),
            component_location_offsets: Some(component_location_offsets),
            change_detection_stats: ChangeDetectionScanStats::default(),
            state: Some(NonNull::from(state)),
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn uses_cached_component_locations(&self) -> bool {
        self.locations.is_some()
            && self.component_locations.is_some()
            && self.component_location_offsets.is_some()
    }
}

impl<'world, 'entities, D, F> Iterator for QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            let index = self.index;
            self.index += 1;
            if let (Some(locations), Some(component_locations), Some(component_location_offsets)) = (
                self.locations,
                self.component_locations,
                self.component_location_offsets,
            ) {
                let stable_location = locations.get(index).copied()?;
                let component_locations = cached_query_component_locations(
                    component_locations,
                    component_location_offsets,
                    index,
                )?;
                let filter_matches = if self.state.is_some() {
                    F::matches_component_locations_with_stats(
                        self.world,
                        entity,
                        component_locations,
                        self.ticks,
                        &mut self.change_detection_stats,
                    )
                } else {
                    F::matches_component_locations(
                        self.world,
                        entity,
                        component_locations,
                        self.ticks,
                    )
                };
                if filter_matches {
                    if let Some(item) = D::fetch_with_component_locations(
                        self.world,
                        entity,
                        stable_location,
                        component_locations,
                        self.ticks,
                    ) {
                        return Some(item);
                    }
                }
                continue;
            }
            if F::matches(self.world, entity, self.ticks) && D::matches_data(self.world, entity) {
                if let Some(item) = D::fetch_with_ticks(self.world, entity, self.ticks) {
                    return Some(item);
                }
            }
        }
        None
    }
}

impl<'world, 'entities, D, F> Drop for QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    fn drop(&mut self) {
        if let Some(state) = self.state {
            // SAFETY: cached constructors derive this pointer from the same
            // QueryState borrow that owns the cached slices held by the iterator.
            let state = unsafe { state.as_ref() };
            state.record_change_detection_stats(self.change_detection_stats);
        }
    }
}
