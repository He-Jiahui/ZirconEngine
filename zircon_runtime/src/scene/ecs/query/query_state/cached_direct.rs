use crate::scene::ecs::{
    CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter, ChangeTickWindow,
    QueryEntityError, QueryEntityItem, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::{cached_query_iter::cached_query_many_indices, single_from_iter};
use super::helpers::collect_many_query_items;
use super::QueryState;

impl<D, F> QueryState<D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    pub fn iter_cached_direct<'world, 'state>(
        &'state mut self,
        world: &'world World,
    ) -> CachedQueryIter<'world, 'state, D, F> {
        self.iter_cached_direct_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub fn single_cached_direct<'world>(
        &mut self,
        world: &'world World,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        single_from_iter(self.iter_cached_direct(world))
    }

    pub fn iter_many_cached_direct<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
    ) -> CachedQueryManyIter<'world, 'state, D, F>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.iter_many_cached_direct_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_many_unique_cached_direct<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> CachedQueryManyIter<'world, 'state, D, F> {
        self.iter_many_unique_cached_direct_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_cached_direct<'world>(
        &mut self,
        world: &'world World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        self.get_cached_direct_with_ticks(
            world,
            entity,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many_cached_direct<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_cached_direct_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many_unique_cached_direct<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_unique_cached_direct_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn is_empty_cached_direct(&mut self, world: &World) -> bool {
        self.is_empty_cached_direct_with_ticks(
            world,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn count_cached_direct(&mut self, world: &World) -> usize {
        self.count_cached_direct_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub fn contains_cached_direct(&mut self, world: &World, entity: EntityId) -> bool {
        self.contains_cached_direct_with_ticks(
            world,
            entity,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub(crate) fn iter_cached_direct_with_ticks<'world, 'state>(
        &'state mut self,
        world: &'world World,
        ticks: ChangeTickWindow,
    ) -> CachedQueryIter<'world, 'state, D, F> {
        self.update_cache(world);
        CachedQueryIter::new(
            world,
            &self.cached_entities,
            &self.cached_locations,
            &self.cached_component_locations,
            ticks,
        )
    }

    pub(crate) fn iter_many_cached_direct_with_ticks<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> CachedQueryManyIter<'world, 'state, D, F>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.update_cache(world);
        let indices = cached_query_many_indices(&self.cached_entity_indices, entities);
        CachedQueryManyIter::new(
            world,
            &self.cached_entities,
            &self.cached_locations,
            &self.cached_component_locations,
            indices,
            ticks,
        )
    }

    pub(crate) fn iter_many_unique_cached_direct_with_ticks<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> CachedQueryManyIter<'world, 'state, D, F> {
        self.iter_many_cached_direct_with_ticks(world, entities, ticks)
    }

    pub(crate) fn is_empty_cached_direct_with_ticks(
        &mut self,
        world: &World,
        ticks: ChangeTickWindow,
    ) -> bool {
        self.iter_cached_direct_with_ticks(world, ticks)
            .next()
            .is_none()
    }

    pub(crate) fn count_cached_direct_with_ticks(
        &mut self,
        world: &World,
        ticks: ChangeTickWindow,
    ) -> usize {
        self.iter_cached_direct_with_ticks(world, ticks).count()
    }

    pub(crate) fn contains_cached_direct_with_ticks(
        &mut self,
        world: &World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        self.update_cache(world);
        let Some(index) = self.cached_entity_index(entity) else {
            return false;
        };
        let component_locations = &self.cached_component_locations[index];
        F::matches_cached(world, entity, component_locations, ticks)
            && D::matches_cached_data(world, entity, component_locations)
    }

    pub(crate) fn get_cached_direct_with_ticks<'world>(
        &mut self,
        world: &'world World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        self.update_cache(world);
        let Some(index) = self.cached_entity_index(entity) else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        let stable_location = self.cached_locations[index];
        let component_locations = &self.cached_component_locations[index];
        if !F::matches_cached(world, entity, component_locations, ticks)
            || !D::matches_cached_data(world, entity, component_locations)
        {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        D::fetch_cached(world, entity, stable_location, component_locations, ticks)
            .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }

    pub(crate) fn get_many_cached_direct_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.update_cache(world);
        collect_many_query_items(entities, |entity| {
            self.get_cached_direct_after_update_with_ticks(world, entity, ticks)
        })
    }

    pub(crate) fn get_many_unique_cached_direct_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_cached_direct_with_ticks(world, entities.into_inner(), ticks)
    }

    fn get_cached_direct_after_update_with_ticks<'world>(
        &self,
        world: &'world World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        let Some(index) = self.cached_entity_index(entity) else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        let stable_location = self.cached_locations[index];
        let component_locations = &self.cached_component_locations[index];
        if !F::matches_cached(world, entity, component_locations, ticks)
            || !D::matches_cached_data(world, entity, component_locations)
        {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        D::fetch_cached(world, entity, stable_location, component_locations, ticks)
            .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }
}
