use std::array;

use crate::scene::ecs::{
    ChangeTickWindow, QueryCombinationMutIter, QueryEntityError, QueryEntityItem, QueryFilter,
    QueryManyMutIter, QueryManyUniqueMutIter, QueryMutData, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

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

    pub fn iter_many_mut<'world, EntityList>(
        &mut self,
        world: &'world mut World,
        entities: EntityList,
    ) -> QueryManyMutIter<'world, D, F, EntityList::IntoIter>
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

    pub fn iter_many_unique_mut<'world, const N: usize>(
        &mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyUniqueMutIter<'world, D, F, array::IntoIter<EntityId, N>> {
        self.iter_many_unique_mut_with_ticks(
            world,
            entities,
            ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    pub fn iter_combinations_mut<'world, const K: usize>(
        &mut self,
        world: &'world mut World,
    ) -> QueryCombinationMutIter<'world, D, F, K> {
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
        D::fetch_mut_with_ticks(world, entity, ticks)
            .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }

    pub(crate) fn single_mut_with_ticks<'world>(
        &mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        self.update_cache(world);
        let mut matched = None;
        for entity in self.cached_entities.iter().copied() {
            if D::matches_data(world, entity) && F::matches(world, entity, ticks) {
                if matched.replace(entity).is_some() {
                    return Err(QuerySingleError::MultipleEntities);
                }
            }
        }

        let entity = matched.ok_or(QuerySingleError::NoEntities)?;
        D::fetch_mut_with_ticks(world, entity, ticks).ok_or(QuerySingleError::NoEntities)
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

    pub(crate) fn iter_many_mut_with_ticks<'world, EntityList>(
        &mut self,
        world: &'world mut World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyMutIter<'world, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        self.update_cache(world);
        QueryManyMutIter::new(world, self.cached_entities.clone(), entities, ticks)
    }

    pub(crate) fn iter_many_unique_mut_with_ticks<'world, const N: usize>(
        &mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyUniqueMutIter<'world, D, F, array::IntoIter<EntityId, N>> {
        self.update_cache(world);
        QueryManyUniqueMutIter::new(world, self.cached_entities.clone(), entities, ticks)
    }

    pub(crate) fn iter_combinations_mut_with_ticks<'world, const K: usize>(
        &mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationMutIter<'world, D, F, K> {
        self.update_cache(world);
        QueryCombinationMutIter::new(world, self.cached_entities.iter().copied(), ticks)
    }

    pub(crate) fn for_each_mut_with_ticks(
        &mut self,
        world: &mut World,
        ticks: ChangeTickWindow,
        mut f: impl FnMut(D::Item<'_>),
    ) {
        self.update_cache(world);
        let entities = self.cached_entities.clone();
        for entity in entities {
            if F::matches(world, entity, ticks) {
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
        if self.cached_entity_index(entity).is_none()
            || !D::matches_data(world, entity)
            || !F::matches(world, entity, ticks)
        {
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
    D::fetch_mut_with_ticks(unsafe { &mut *world }, entity, ticks)
        .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
}
