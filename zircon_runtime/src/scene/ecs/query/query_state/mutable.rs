use std::array;

use crate::scene::ecs::{
    ChangeTickWindow, QueryCombinationMutIter, QueryEntityError, QueryEntityItem, QueryFilter,
    QueryManyMutIter, QueryManyUniqueMutIter, QueryMutData, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::cached_query_iter::cached_query_component_locations;
use super::super::unique_entities::first_duplicate_entity;
use super::helpers::collect_many_query_items;
use super::QueryState;

impl<D, F> QueryState<D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn get_mut<'world>(
        &mut self,
        world: &'world mut World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        self.get_mut_with_ticks(
            world,
            entity,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn single_mut<'world>(
        &mut self,
        world: &'world mut World,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        self.single_mut_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub fn get_many_mut<'world, const N: usize>(
        &mut self,
        world: &'world mut World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_mut_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn get_many_unique_mut<'world, const N: usize>(
        &mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_unique_mut_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_many_mut<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world mut World,
        entities: EntityList,
    ) -> QueryManyMutIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.iter_many_mut_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_many_unique_mut<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyUniqueMutIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        self.iter_many_unique_mut_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_combinations_mut<'world, 'state, const K: usize>(
        &'state mut self,
        world: &'world mut World,
    ) -> QueryCombinationMutIter<'world, 'state, D, F, K> {
        self.iter_combinations_mut_with_ticks(
            world,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn for_each_mut(&mut self, world: &mut World, f: impl FnMut(D::Item<'_>)) {
        let ticks = ChangeTickWindow::all(world.read_change_tick());
        self.for_each_mut_with_ticks(world, ticks, f);
    }

    pub(crate) fn get_mut_with_ticks<'world>(
        &mut self,
        world: &'world mut World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        self.update_cache(world);
        self.validate_mut_after_update_with_ticks(world, entity, ticks)?;
        let Some(item) = D::fetch_mut_with_ticks(world, entity, ticks) else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        Ok(item)
    }

    pub(crate) fn single_mut_with_ticks<'world>(
        &mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        self.update_cache(world);
        let mut matched = None;
        for (index, entity) in self.cached_entities.iter().copied().enumerate() {
            let Some(component_locations) = cached_query_component_locations(
                &self.cached_component_locations,
                &self.cached_component_location_offsets,
                index,
            ) else {
                continue;
            };
            if F::matches_component_locations(world, entity, component_locations, ticks) {
                if matched.replace(entity).is_some() {
                    return Err(QuerySingleError::MultipleEntities);
                }
            }
        }

        let Some(entity) = matched else {
            return Err(QuerySingleError::NoEntities);
        };
        let Some(item) = D::fetch_mut_with_ticks(world, entity, ticks) else {
            return Err(QuerySingleError::NoEntities);
        };
        Ok(item)
    }

    pub(crate) fn get_many_mut_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world mut World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        if let Some(entity) = first_duplicate_entity(&entities) {
            return Err(QueryEntityError::AliasedMutability(entity));
        }
        self.update_cache(world);
        for entity in entities.iter().copied() {
            self.validate_mut_after_update_with_ticks(world, entity, ticks)?;
        }

        let world = world as *mut World;
        collect_many_query_items(entities, |entity| {
            // Duplicate IDs were rejected above and the query access descriptor
            // guarantees one mutable data shape, so each returned item is from a
            // distinct entity.
            unsafe { fetch_mut_after_validation_unchecked::<D>(world, entity, ticks) }
        })
    }

    pub(crate) fn get_many_unique_mut_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        self.get_many_mut_with_ticks(world, entities.into_inner(), ticks)
    }

    pub(crate) fn iter_many_mut_with_ticks<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world mut World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyMutIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.update_cache(world);
        QueryManyMutIter::new(
            world,
            &self.cached_entity_indices,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            entities,
            ticks,
        )
    }

    pub(crate) fn iter_many_unique_mut_with_ticks<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyUniqueMutIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        self.update_cache(world);
        QueryManyUniqueMutIter::new(
            world,
            &self.cached_entity_indices,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            entities,
            ticks,
        )
    }

    pub(crate) fn iter_combinations_mut_with_ticks<'world, 'state, const K: usize>(
        &'state mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationMutIter<'world, 'state, D, F, K> {
        self.update_cache(world);
        QueryCombinationMutIter::new_from_cached_entities(
            world,
            &self.cached_entities,
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            ticks,
        )
    }

    pub(crate) fn for_each_mut_with_ticks(
        &mut self,
        world: &mut World,
        ticks: ChangeTickWindow,
        mut f: impl FnMut(D::Item<'_>),
    ) {
        self.update_cache(world);
        for (index, entity) in self.cached_entities.iter().copied().enumerate() {
            let Some(component_locations) = cached_query_component_locations(
                &self.cached_component_locations,
                &self.cached_component_location_offsets,
                index,
            ) else {
                continue;
            };
            if F::matches_component_locations(world, entity, component_locations, ticks) {
                if let Some(item) = D::fetch_mut_with_ticks(world, entity, ticks) {
                    f(item);
                }
            }
        }
    }

    fn validate_mut_after_update_with_ticks(
        &self,
        world: &World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<(), QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        let Some((_, component_locations)) = self.cached_entity_location(entity) else {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        };
        if !F::matches_component_locations(world, entity, component_locations, ticks) {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        Ok(())
    }
}

unsafe fn fetch_mut_after_validation_unchecked<'world, D>(
    world: *mut World,
    entity: EntityId,
    ticks: ChangeTickWindow,
) -> Result<D::Item<'world>, QueryEntityError>
where
    D: QueryMutData,
{
    let Some(item) = D::fetch_mut_with_ticks(unsafe { &mut *world }, entity, ticks) else {
        return Err(QueryEntityError::QueryDoesNotMatch(entity));
    };
    Ok(item)
}
