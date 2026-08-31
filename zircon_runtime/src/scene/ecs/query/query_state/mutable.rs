use std::array;

use crate::scene::EntityId;
use crate::scene::World;
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryCombinationMutIter, QueryEntityError,
    QueryEntityItem, QueryFilter, QueryManyMutIter, QueryManyUniqueMutIter, QueryMutData,
    QuerySingleError, UniqueEntityArray,
};

use super::super::unique_entities::first_duplicate_entity;
use super::many_item_array::collect_many_query_items;
use super::{CachedArchetypePlan, QueryState, project_entity_from_plans};

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
        let mut component_locations = Vec::with_capacity(self.access.reads().len());
        self.validate_entity_with_locations(world, entity, ticks, &mut component_locations)?;
        D::fetch_mut_with_component_locations(world, entity, &component_locations, ticks)
            .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
    }

    pub(crate) fn single_mut_with_ticks<'world>(
        &mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        self.update_cache(world);
        let mut component_locations = Vec::with_capacity(self.access.reads().len());
        let mut matched = None;
        for stable_location in world.stable_query_location_iter(
            self.cached_archetype_plans
                .iter()
                .map(CachedArchetypePlan::archetype_id),
        ) {
            let entity = stable_location.stable_id;
            if self
                .validate_entity_with_locations(world, entity, ticks, &mut component_locations)
                .is_ok()
                && matched.replace(entity).is_some()
            {
                return Err(QuerySingleError::MultipleEntities);
            }
        }

        let Some(entity) = matched else {
            return Err(QuerySingleError::NoEntities);
        };
        component_locations.clear();
        self.validate_entity_with_locations(world, entity, ticks, &mut component_locations)
            .map_err(|_| QuerySingleError::NoEntities)?;
        D::fetch_mut_with_component_locations(world, entity, &component_locations, ticks)
            .ok_or(QuerySingleError::NoEntities)
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
        let mut component_locations = Vec::with_capacity(self.access.reads().len());
        for entity in entities.iter().copied() {
            self.validate_entity_with_locations(world, entity, ticks, &mut component_locations)?;
        }

        let world = world as *mut World;
        let plans = &self.cached_archetype_plans;
        collect_many_query_items(entities, |entity| {
            // Duplicate IDs were rejected above, so each returned mutable item
            // belongs to a distinct entity.
            unsafe { fetch_mut_from_plans_unchecked::<D>(world, plans, entity, ticks) }
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
        QueryManyMutIter::new(world, &self.cached_archetype_plans, entities, ticks)
    }

    pub(crate) fn iter_many_unique_mut_with_ticks<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world mut World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyUniqueMutIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        self.update_cache(world);
        QueryManyUniqueMutIter::new(world, &self.cached_archetype_plans, entities, ticks)
    }

    pub(crate) fn iter_combinations_mut_with_ticks<'world, 'state, const K: usize>(
        &'state mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationMutIter<'world, 'state, D, F, K> {
        self.update_cache(world);
        QueryCombinationMutIter::new_from_cached_plans(world, &self.cached_archetype_plans, ticks)
    }

    pub(crate) fn for_each_mut_with_ticks(
        &mut self,
        world: &mut World,
        ticks: ChangeTickWindow,
        mut f: impl FnMut(D::Item<'_>),
    ) {
        self.update_cache(world);
        let candidates = world
            .stable_query_location_iter(
                self.cached_archetype_plans
                    .iter()
                    .map(CachedArchetypePlan::archetype_id),
            )
            .collect::<Vec<_>>();
        let world = world as *mut World;
        for stable_location in candidates {
            let entity = stable_location.stable_id;
            let mut component_locations = Vec::new();
            let shared_world = unsafe { &*world };
            if project_entity_from_plans(
                &self.cached_archetype_plans,
                shared_world,
                entity,
                &mut component_locations,
            )
            .is_none()
                || !F::matches_component_locations(
                    shared_world,
                    entity,
                    &component_locations,
                    ticks,
                )
            {
                continue;
            }
            let item = unsafe {
                D::fetch_mut_with_component_locations(
                    &mut *world,
                    entity,
                    &component_locations,
                    ticks,
                )
            };
            if let Some(item) = item {
                f(item);
            }
        }
    }

    fn validate_entity_with_locations(
        &self,
        world: &World,
        entity: EntityId,
        ticks: ChangeTickWindow,
        component_locations: &mut Vec<ComponentStorageLocation>,
    ) -> Result<(), QueryEntityError> {
        if !world.contains_entity(entity) {
            return Err(QueryEntityError::NotSpawned(entity));
        }
        if project_entity_from_plans(
            &self.cached_archetype_plans,
            world,
            entity,
            component_locations,
        )
        .is_none()
            || !F::matches_component_locations(world, entity, component_locations, ticks)
        {
            return Err(QueryEntityError::QueryDoesNotMatch(entity));
        }
        Ok(())
    }
}

unsafe fn fetch_mut_from_plans_unchecked<'world, D>(
    world: *mut World,
    plans: &[CachedArchetypePlan],
    entity: EntityId,
    ticks: ChangeTickWindow,
) -> Result<D::Item<'world>, QueryEntityError>
where
    D: QueryMutData,
{
    let mut component_locations = Vec::new();
    let shared_world = unsafe { &*world };
    if project_entity_from_plans(plans, shared_world, entity, &mut component_locations).is_none() {
        return Err(QueryEntityError::QueryDoesNotMatch(entity));
    }
    D::fetch_mut_with_component_locations(
        unsafe { &mut *world },
        entity,
        &component_locations,
        ticks,
    )
    .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
}
