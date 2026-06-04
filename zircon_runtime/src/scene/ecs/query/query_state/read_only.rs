use std::array;

use crate::scene::ecs::{
    ChangeTickWindow, QueryCombinationIter, QueryData, QueryEntityError, QueryEntityItem,
    QueryFilter, QueryIter, QueryManyIter, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::single_from_iter;
use super::helpers::{cached_many_entities, collect_many_query_items};
use super::QueryState;

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

    pub fn single<'world>(
        &self,
        world: &'world World,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        single_from_iter(self.iter(world))
    }

    pub fn iter_many<'world, EntityList>(
        &self,
        world: &'world World,
        entities: EntityList,
    ) -> QueryManyIter<'world, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.iter_many_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_many_unique<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyIter<'world, D, F, array::IntoIter<EntityId, N>> {
        self.iter_many_unique_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_many_cached<'world, EntityList>(
        &mut self,
        world: &'world World,
        entities: EntityList,
    ) -> QueryManyIter<'world, D, F>
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

    pub fn iter_many_unique_cached<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyIter<'world, D, F> {
        self.iter_many_unique_cached_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_combinations<'world, const K: usize>(
        &self,
        world: &'world World,
    ) -> QueryCombinationIter<'world, D, F, K> {
        self.iter_combinations_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub fn iter_combinations_cached<'world, const K: usize>(
        &mut self,
        world: &'world World,
    ) -> QueryCombinationIter<'world, D, F, K> {
        self.iter_combinations_cached_with_ticks(
            world,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get<'world>(
        &self,
        world: &'world World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        self.get_with_ticks(
            world,
            entity,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many_unique<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_unique_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn is_empty(&self, world: &World) -> bool {
        self.iter(world).next().is_none()
    }

    pub fn count(&self, world: &World) -> usize {
        self.iter(world).count()
    }

    pub fn contains(&self, world: &World, entity: EntityId) -> bool {
        self.contains_with_ticks(
            world,
            entity,
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

    pub(crate) fn contains_with_ticks(
        &self,
        world: &World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        world.contains_entity(entity)
            && D::matches_data(world, entity)
            && F::matches(world, entity, ticks)
    }

    pub(crate) fn get_with_ticks<'world>(
        &self,
        world: &'world World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        if !D::matches_data(world, entity) || !F::matches(world, entity, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        D::fetch_with_ticks(world, entity, ticks).ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }

    pub(crate) fn get_many_with_ticks<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        collect_many_query_items(entities, |entity| self.get_with_ticks(world, entity, ticks))
    }

    pub(crate) fn get_many_unique_with_ticks<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_with_ticks(world, entities.into_inner(), ticks)
    }

    pub(crate) fn iter_many_unique_with_ticks<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyIter<'world, D, F, array::IntoIter<EntityId, N>> {
        self.iter_many_with_ticks(world, entities, ticks)
    }

    pub(crate) fn iter_many_with_ticks<'world, EntityList>(
        &self,
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyIter<'world, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        QueryManyIter::new(world, entities, ticks)
    }

    pub(crate) fn iter_many_unique_cached_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyIter<'world, D, F> {
        self.iter_many_cached_with_ticks(world, entities, ticks)
    }

    pub(crate) fn iter_combinations_with_ticks<'world, const K: usize>(
        &self,
        world: &'world World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationIter<'world, D, F, K> {
        QueryCombinationIter::new(world, world.entity_ids_for_query().iter().copied(), ticks)
    }

    pub(crate) fn iter_combinations_cached_with_ticks<'world, const K: usize>(
        &mut self,
        world: &'world World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationIter<'world, D, F, K> {
        self.update_cache(world);
        QueryCombinationIter::new(world, self.cached_entities.iter().copied(), ticks)
    }

    pub(crate) fn iter_many_cached_with_ticks<'world, EntityList>(
        &mut self,
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyIter<'world, D, F>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.update_cache(world);
        let entities = cached_many_entities(&self.cached_entity_indices, entities);
        QueryManyIter::new(world, entities, ticks)
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
        self.cached_entity_index(entity).is_some() && F::matches(world, entity, ticks)
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
        if self.cached_entity_index(entity).is_none() || !F::matches(world, entity, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        D::fetch_with_ticks(world, entity, ticks).ok_or(QueryEntityError::QueryDoesNotMatch(entity))
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

    fn get_cached_after_update_with_ticks<'world>(
        &self,
        world: &'world World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        if self.cached_entity_index(entity).is_none() || !F::matches(world, entity, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        D::fetch_with_ticks(world, entity, ticks).ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }
}
