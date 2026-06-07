use std::array;

use crate::scene::ecs::{
    ChangeTickWindow, QueryCombinationIter, QueryData, QueryEntityError, QueryEntityItem,
    QueryFilter, QueryIter, QueryManyIter, QuerySingleError, UniqueEntityArray,
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

    pub fn iter_combinations<'world, 'state, const K: usize>(
        &'state self,
        world: &'world World,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        self.iter_combinations_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
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

    pub(crate) fn iter_combinations_with_ticks<'world, 'state, const K: usize>(
        &'state self,
        world: &'world World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        QueryCombinationIter::new(world, world.entity_ids_for_query().iter().copied(), ticks)
    }
}
