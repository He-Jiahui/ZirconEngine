use crate::scene::ecs::{
    CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter, ChangeTickWindow,
    QueryEntityError, QueryEntityItem, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::{cached_query_iter::cached_query_component_locations, single_from_iter};
use super::many_item_array::collect_many_query_items;
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
    ) -> CachedQueryManyIter<'world, 'state, D, F, EntityList::IntoIter>
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
    ) -> CachedQueryManyIter<'world, 'state, D, F, std::array::IntoIter<EntityId, N>> {
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
            &self.cached_component_location_offsets,
            ticks,
        )
    }

    pub(crate) fn iter_many_cached_direct_with_ticks<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> CachedQueryManyIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.update_cache(world);
        CachedQueryManyIter::new(
            world,
            &self.cached_entities,
            &self.cached_locations,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            &self.cached_entity_indices,
            entities,
            ticks,
        )
    }

    pub(crate) fn iter_many_unique_cached_direct_with_ticks<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> CachedQueryManyIter<'world, 'state, D, F, std::array::IntoIter<EntityId, N>> {
        self.iter_many_cached_direct_with_ticks(world, entities, ticks)
    }

    pub(crate) fn is_empty_cached_direct_with_ticks(
        &mut self,
        world: &World,
        ticks: ChangeTickWindow,
    ) -> bool {
        self.update_cache(world);
        let mut index = 0_usize;
        while index < self.cached_entities.len() {
            let entity = self.cached_entities[index];
            let Some(stable_location) = self.cached_locations.get(index).copied() else {
                return true;
            };
            let Some(component_locations) = cached_query_component_locations(
                &self.cached_component_locations,
                &self.cached_component_location_offsets,
                index,
            ) else {
                return true;
            };
            if F::matches_cached(world, entity, component_locations, ticks)
                && D::fetch_cached(world, entity, stable_location, component_locations, ticks)
                    .is_some()
            {
                return false;
            }
            index += 1;
        }
        true
    }

    pub(crate) fn count_cached_direct_with_ticks(
        &mut self,
        world: &World,
        ticks: ChangeTickWindow,
    ) -> usize {
        self.update_cache(world);
        let mut count = 0_usize;
        let mut index = 0_usize;
        while index < self.cached_entities.len() {
            let entity = self.cached_entities[index];
            let Some(stable_location) = self.cached_locations.get(index).copied() else {
                return count;
            };
            let Some(component_locations) = cached_query_component_locations(
                &self.cached_component_locations,
                &self.cached_component_location_offsets,
                index,
            ) else {
                return count;
            };
            if F::matches_cached(world, entity, component_locations, ticks)
                && D::fetch_cached(world, entity, stable_location, component_locations, ticks)
                    .is_some()
            {
                count += 1;
            }
            index += 1;
        }
        count
    }

    pub(crate) fn contains_cached_direct_with_ticks(
        &mut self,
        world: &World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        self.update_cache(world);
        let Some((_, component_locations)) = self.cached_entity_location(entity) else {
            return false;
        };
        F::matches_cached(world, entity, component_locations, ticks)
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
        let Some((stable_location, component_locations)) = self.cached_entity_location(entity)
        else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        if !F::matches_cached(world, entity, component_locations, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        let Some(item) =
            D::fetch_cached(world, entity, stable_location, component_locations, ticks)
        else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        Ok(item)
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
        let Some((stable_location, component_locations)) = self.cached_entity_location(entity)
        else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        if !F::matches_cached(world, entity, component_locations, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        let Some(item) =
            D::fetch_cached(world, entity, stable_location, component_locations, ticks)
        else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        Ok(item)
    }
}
