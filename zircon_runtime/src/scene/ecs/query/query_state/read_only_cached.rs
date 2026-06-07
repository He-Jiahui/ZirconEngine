use std::array;

use crate::scene::ecs::{
    ChangeTickWindow, QueryCombinationIter, QueryData, QueryEntityError, QueryEntityItem,
    QueryFilter, QueryIter, QueryManyCachedIter, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::single_from_iter;
use super::helpers::collect_many_query_items;
use super::QueryState;

impl<D, F> QueryState<D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub fn iter_many_cached<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
    ) -> QueryManyCachedIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.iter_many_cached_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_many_unique_cached<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyCachedIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        self.iter_many_unique_cached_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_combinations_cached<'world, 'state, const K: usize>(
        &'state mut self,
        world: &'world World,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        self.iter_combinations_cached_with_ticks(
            world,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_cached<'world, 'state>(
        &'state mut self,
        world: &'world World,
    ) -> QueryIter<'world, 'state, D, F> {
        self.iter_cached_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub(crate) fn iter_cached_with_ticks<'world, 'state>(
        &'state mut self,
        world: &'world World,
        ticks: ChangeTickWindow,
    ) -> QueryIter<'world, 'state, D, F> {
        self.update_cache(world);
        QueryIter::new_cached_locations(
            world,
            &self.cached_entities,
            &self.cached_locations,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            ticks,
        )
    }

    pub fn single_cached<'world>(
        &mut self,
        world: &'world World,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        single_from_iter(self.iter_cached(world))
    }

    pub fn get_cached<'world>(
        &mut self,
        world: &'world World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        self.get_cached_with_ticks(
            world,
            entity,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many_cached<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_cached_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many_unique_cached<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_unique_cached_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn is_empty_cached(&mut self, world: &World) -> bool {
        self.is_empty_cached_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub fn count_cached(&mut self, world: &World) -> usize {
        self.count_cached_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub fn contains_cached(&mut self, world: &World, entity: EntityId) -> bool {
        self.contains_cached_with_ticks(
            world,
            entity,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub(crate) fn iter_many_unique_cached_with_ticks<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyCachedIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        self.iter_many_cached_with_ticks(world, entities, ticks)
    }

    pub(crate) fn iter_combinations_cached_with_ticks<'world, 'state, const K: usize>(
        &'state mut self,
        world: &'world World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        self.update_cache(world);
        QueryCombinationIter::new_from_cached_entities(
            world,
            &self.cached_entities,
            &self.cached_locations,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            ticks,
        )
    }

    pub(crate) fn iter_many_cached_with_ticks<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyCachedIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.update_cache(world);
        QueryManyCachedIter::new(
            world,
            &self.cached_entity_indices,
            &self.cached_locations,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            entities,
            ticks,
        )
    }

    pub(crate) fn is_empty_cached_with_ticks(
        &mut self,
        world: &World,
        ticks: ChangeTickWindow,
    ) -> bool {
        self.iter_cached_with_ticks(world, ticks).next().is_none()
    }

    pub(crate) fn count_cached_with_ticks(
        &mut self,
        world: &World,
        ticks: ChangeTickWindow,
    ) -> usize {
        self.iter_cached_with_ticks(world, ticks).count()
    }

    pub(crate) fn contains_cached_with_ticks(
        &mut self,
        world: &World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        self.update_cache(world);
        let Some((_, component_locations)) = self.cached_entity_location(entity) else {
            return false;
        };
        F::matches_component_locations(world, entity, component_locations, ticks)
    }

    pub(crate) fn get_cached_with_ticks<'world>(
        &mut self,
        world: &'world World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        self.update_cache(world);
        self.get_cached_after_update_with_ticks(world, entity, ticks)
    }

    fn get_cached_after_update_with_ticks<'world>(
        &self,
        world: &'world World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        let Some((stable_location, component_locations)) = self.cached_entity_location(entity)
        else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        if !F::matches_component_locations(world, entity, component_locations, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        D::fetch_with_component_locations(
            world,
            entity,
            stable_location,
            component_locations,
            ticks,
        )
        .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }

    pub(crate) fn get_many_cached_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.update_cache(world);
        collect_many_query_items(entities, |entity| {
            self.get_cached_after_update_with_ticks(world, entity, ticks)
        })
    }

    pub(crate) fn get_many_unique_cached_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_cached_with_ticks(world, entities.into_inner(), ticks)
    }
}
