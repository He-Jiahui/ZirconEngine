use std::{marker::PhantomData, ptr::NonNull};

use super::query_state::{CachedArchetypePlan, find_cached_archetype_plan};
use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter,
    QueryState,
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
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
    change_detection_stats: ChangeDetectionScanStats,
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
        plans: &'state [CachedArchetypePlan],
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
            plans,
            component_locations: Vec::new(),
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
        for entity_item in self.entities.by_ref() {
            let entity = entity_item.entity_id();
            let Some(stable_location) = self.world.internal_entity_location(entity) else {
                continue;
            };
            let Some(plan) =
                find_cached_archetype_plan(self.plans, stable_location.location.archetype_id)
            else {
                continue;
            };
            if !plan.write_component_locations(
                self.world,
                stable_location,
                &mut self.component_locations,
            ) {
                continue;
            }
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
            // SAFETY: the pointer originates from the QueryState borrow that owns
            // the plans held by this iterator.
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
