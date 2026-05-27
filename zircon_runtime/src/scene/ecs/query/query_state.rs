use std::{array, marker::PhantomData, mem::MaybeUninit};

use crate::scene::ecs::{
    ArchetypeId, CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter,
    ChangeTickWindow, ComponentStorageLocation, QueryAccess, QueryAccessError,
    QueryCombinationIter, QueryCombinationMutIter, QueryData, QueryDataAccess, QueryEntityError,
    QueryEntityItem, QueryFilter, QueryIter, QueryManyIter, QueryManyMutIter,
    QueryManyUniqueMutIter, QueryMutData, QuerySingleError, StableEntityLocation, SystemParam,
    SystemParamAccess, SystemParamError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::{
    cached_query_iter::{cached_query_entity_index, cached_query_many_indices},
    single_from_iter,
    unique_entities::first_duplicate_entity,
};

#[derive(Clone, Debug)]
pub struct QueryState<D, F = ()> {
    access: QueryAccess,
    cached_archetypes: Vec<ArchetypeId>,
    cached_archetype_generation: u64,
    cached_entities: Vec<EntityId>,
    cached_entity_indices: Vec<(EntityId, usize)>,
    cached_locations: Vec<StableEntityLocation>,
    cached_component_locations: Vec<Vec<ComponentStorageLocation>>,
    cached_revision: u64,
    cache_rebuilds: u64,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<D, F> QueryState<D, F>
where
    D: QueryDataAccess,
    F: QueryFilter,
{
    pub fn new(world: &mut World) -> Self {
        Self::try_new(world).expect("query data must not request conflicting component access")
    }

    pub fn try_new(world: &mut World) -> Result<Self, QueryAccessError> {
        let mut access = QueryAccess::default();
        D::update_access(world, &mut access)?;
        F::update_access(world, &mut access)?;
        let mut state = Self {
            access,
            cached_archetypes: Vec::new(),
            cached_archetype_generation: 0,
            cached_entities: Vec::new(),
            cached_entity_indices: Vec::new(),
            cached_locations: Vec::new(),
            cached_component_locations: Vec::new(),
            cached_revision: u64::MAX,
            cache_rebuilds: 0,
            _marker: PhantomData,
        };
        state.update_cache(world);
        Ok(state)
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn conflicts_with<OtherD, OtherF>(&self, other: &QueryState<OtherD, OtherF>) -> bool {
        self.access.conflicts_with(&other.access)
    }

    pub fn update_cache(&mut self, world: &World) {
        let revision = world.query_cache_revision();
        if self.cached_revision == revision {
            return;
        }
        self.cached_entities.clear();
        self.cached_entity_indices.clear();
        self.cached_locations.clear();
        self.cached_component_locations.clear();
        let mut cached_component_ids = self.access.reads().to_vec();
        for component_id in self.access.writes().iter().copied() {
            if !cached_component_ids.contains(&component_id) {
                cached_component_ids.push(component_id);
            }
        }
        let (matched_archetypes, candidate_locations) =
            world.entity_locations_matching_query_archetypes(&self.access);
        self.cached_archetypes = matched_archetypes;
        self.cached_archetype_generation = world.archetype_generation();
        for location in candidate_locations {
            let component_locations = world
                .component_storage_locations_for_internal(location.internal, &cached_component_ids);
            if D::matches_component_locations(world, location.stable_id, &component_locations) {
                let cache_index = self.cached_entities.len();
                self.cached_entities.push(location.stable_id);
                self.cached_entity_indices
                    .push((location.stable_id, cache_index));
                self.cached_locations.push(location);
                self.cached_component_locations.push(component_locations);
            }
        }
        self.cached_entity_indices
            .sort_unstable_by_key(|(entity, _)| *entity);
        self.cached_revision = revision;
        self.cache_rebuilds = self.cache_rebuilds.saturating_add(1);
    }

    pub fn cached_archetype_count(&self) -> usize {
        self.cached_archetypes.len()
    }

    pub fn cached_archetype_generation(&self) -> u64 {
        self.cached_archetype_generation
    }

    pub fn cached_entity_count(&self) -> usize {
        self.cached_entities.len()
    }

    pub(crate) fn cached_entity_index(&self, entity: EntityId) -> Option<usize> {
        cached_query_entity_index(&self.cached_entity_indices, entity)
    }

    pub fn cached_location_count(&self) -> usize {
        self.cached_locations.len()
    }

    pub fn cached_locations(&self) -> &[StableEntityLocation] {
        &self.cached_locations
    }

    pub fn cached_component_locations(&self) -> &[Vec<ComponentStorageLocation>] {
        &self.cached_component_locations
    }

    pub fn cached_revision(&self) -> u64 {
        self.cached_revision
    }

    pub fn cache_rebuilds(&self) -> u64 {
        self.cache_rebuilds
    }
}

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

fn cached_many_entities<EntityList>(
    cached_entity_indices: &[(EntityId, usize)],
    entities: EntityList,
) -> Vec<EntityId>
where
    EntityList: IntoIterator,
    EntityList::Item: QueryEntityItem,
{
    entities
        .into_iter()
        .map(QueryEntityItem::entity_id)
        .filter(|entity| cached_query_entity_index(cached_entity_indices, *entity).is_some())
        .collect()
}

fn collect_many_query_items<Item, const N: usize>(
    entities: [EntityId; N],
    mut get_item: impl FnMut(EntityId) -> Result<Item, QueryEntityError>,
) -> Result<[Item; N], QueryEntityError> {
    let mut values: [MaybeUninit<Item>; N] = std::array::from_fn(|_| MaybeUninit::uninit());
    let mut initialized = 0;

    for (slot, entity) in values.iter_mut().zip(entities) {
        match get_item(entity) {
            Ok(item) => {
                slot.write(item);
                initialized += 1;
            }
            Err(error) => {
                for value in &mut values[..initialized] {
                    // Only slots written before the error contain initialized values.
                    unsafe {
                        value.assume_init_drop();
                    }
                }
                return Err(error);
            }
        }
    }

    Ok(values.map(|value| {
        // Every slot was written by the loop above.
        unsafe { value.assume_init() }
    }))
}

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

impl<D, F> SystemParam for QueryState<D, F>
where
    D: QueryDataAccess + 'static,
    F: QueryFilter,
{
    type State = QueryState<D, F>;
    type Item<'world> = crate::scene::ecs::Query<'world, D, F>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        let state = QueryState::<D, F>::try_new(world)?;
        access.add_query_access(state.access())?;
        Ok(state)
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        crate::scene::ecs::Query::new(world, state, ticks)
    }
}
