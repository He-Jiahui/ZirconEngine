mod cached_direct;
mod helpers;
mod mutable;
mod read_only;
mod system_param;

use std::marker::PhantomData;

use crate::scene::ecs::{
    ArchetypeId, ComponentStorageLocation, QueryAccess, QueryAccessError, QueryDataAccess,
    QueryFilter, StableEntityLocation,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::cached_query_iter::cached_query_entity_index;

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
