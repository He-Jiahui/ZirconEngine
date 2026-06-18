use std::{marker::PhantomData, ptr::NonNull};

use super::cached_query_iter::{cached_query_component_locations, cached_query_entity_index};
use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter,
    QueryState, StableEntityLocation,
};
use crate::scene::{EntityId, World};

pub trait QueryEntityItem {
    fn entity_id(self) -> EntityId;
}

impl QueryEntityItem for EntityId {
    fn entity_id(self) -> EntityId {
        self
    }
}

impl QueryEntityItem for &EntityId {
    fn entity_id(self) -> EntityId {
        *self
    }
}

pub struct QueryManyIter<'world, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: &'world World,
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

pub struct QueryManyCachedIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: &'world World,
    cached_entity_indices: &'state [(EntityId, usize)],
    cached_locations: &'state [StableEntityLocation],
    cached_component_locations: &'state [ComponentStorageLocation],
    cached_component_location_offsets: &'state [usize],
    change_detection_stats: ChangeDetectionScanStats,
    // Cached slices keep the originating QueryState alive; the raw sink avoids
    // imposing an extra state-reference lifetime on query item output.
    state: Option<NonNull<QueryState<D, F>>>,
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, D, F, I> QueryManyIter<'world, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, 'state, D, F, I> QueryManyCachedIter<'world, 'state, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world World,
        cached_entity_indices: &'state [(EntityId, usize)],
        cached_locations: &'state [StableEntityLocation],
        cached_component_locations: &'state [ComponentStorageLocation],
        cached_component_location_offsets: &'state [usize],
        entities: EntityList,
        ticks: ChangeTickWindow,
        state: &'state QueryState<D, F>,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            cached_entity_indices,
            cached_locations,
            cached_component_locations,
            cached_component_location_offsets,
            change_detection_stats: ChangeDetectionScanStats::default(),
            state: Some(NonNull::from(state)),
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, D, F, I> Iterator for QueryManyIter<'world, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        for entity_item in self.entities.by_ref() {
            let entity = entity_item.entity_id();
            if world_entity_matches::<D, F>(self.world, entity, self.ticks) {
                if let Some(item) = D::fetch_with_ticks(self.world, entity, self.ticks) {
                    return Some(item);
                }
            }
        }
        None
    }
}

impl<'world, 'state, D, F, I> Iterator for QueryManyCachedIter<'world, 'state, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        let cached_entity_indices = self.cached_entity_indices;
        let cached_locations = self.cached_locations;
        let cached_component_locations = self.cached_component_locations;
        let cached_component_location_offsets = self.cached_component_location_offsets;
        let ticks = self.ticks;
        let world = self.world;

        for entity_item in self.entities.by_ref() {
            let entity = entity_item.entity_id();
            let Some(index) = cached_query_entity_index(cached_entity_indices, entity) else {
                continue;
            };
            let stable_location = cached_locations.get(index).copied()?;
            let component_locations = cached_query_component_locations(
                cached_component_locations,
                cached_component_location_offsets,
                index,
            )?;
            let filter_matches = if self.state.is_some() {
                F::matches_component_locations_with_stats(
                    world,
                    entity,
                    component_locations,
                    ticks,
                    &mut self.change_detection_stats,
                )
            } else {
                F::matches_component_locations(world, entity, component_locations, ticks)
            };
            if filter_matches {
                if let Some(item) = D::fetch_with_component_locations(
                    world,
                    entity,
                    stable_location,
                    component_locations,
                    ticks,
                ) {
                    return Some(item);
                }
            }
        }
        None
    }
}

impl<'world, 'state, D, F, I> Drop for QueryManyCachedIter<'world, 'state, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
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

fn world_entity_matches<D, F>(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool
where
    D: QueryData,
    F: QueryFilter,
{
    world.contains_entity(entity)
        && F::matches(world, entity, ticks)
        && D::matches_data(world, entity)
}
