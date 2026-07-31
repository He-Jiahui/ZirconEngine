use crate::scene::EntityId;
use crate::scene::World;
use crate::scene::ecs::{
    ComponentStorageLocation, QueryDataAccess, QueryFilter, StableEntityLocation,
};

use super::super::cached_query_iter::{
    cached_query_component_locations, cached_query_entity_index,
};
use super::QueryState;

impl<D, F> QueryState<D, F>
where
    D: QueryDataAccess,
    F: QueryFilter,
{
    pub fn update_cache(&mut self, world: &World) {
        let revision = world.query_cache_revision();
        if self.cached_revision == revision {
            self.cache_hits = self.cache_hits.saturating_add(1);
            return;
        }
        self.cached_entities.clear();
        self.cached_entity_indices.clear();
        self.cached_locations.clear();
        self.cached_component_locations.clear();
        self.cached_component_location_offsets.clear();
        self.cached_component_location_offsets.push(0);
        let matched_archetypes = world.matching_query_archetypes(&self.access);
        let candidate_count = world.matching_query_archetype_entity_count(&matched_archetypes);
        self.cached_archetype_generation = world.archetype_generation();
        let component_count = self.access.reads().len();
        self.cached_entities.reserve(candidate_count);
        self.cached_entity_indices.reserve(candidate_count);
        self.cached_locations.reserve(candidate_count);
        self.cached_component_location_offsets
            .reserve(candidate_count);
        self.cached_component_locations
            .reserve(candidate_count.saturating_mul(component_count));
        let mut component_locations = Vec::with_capacity(component_count);
        world.visit_entity_locations_matching_archetypes(&matched_archetypes, |location| {
            world.component_storage_locations_for_internal(
                location.internal,
                self.access.reads(),
                &mut component_locations,
            );
            if D::matches_component_locations(world, location.stable_id, &component_locations) {
                let cache_index = self.cached_entities.len();
                self.cached_entities.push(location.stable_id);
                self.cached_entity_indices
                    .push((location.stable_id, cache_index));
                self.cached_locations.push(location);
                self.cached_component_locations
                    .extend(component_locations.iter().copied());
                self.cached_component_location_offsets
                    .push(self.cached_component_locations.len());
            }
        });
        self.cached_archetypes = matched_archetypes;
        self.cached_entity_indices
            .sort_unstable_by_key(|(entity, _)| *entity);
        self.cached_revision = revision;
        self.cache_misses = self.cache_misses.saturating_add(1);
        self.cache_rebuilds = self.cache_rebuilds.saturating_add(1);
        self.last_candidate_entity_count = candidate_count;
        self.last_matched_entity_count = self.cached_entities.len();
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

    pub(crate) fn cached_entities(&self) -> &[EntityId] {
        &self.cached_entities
    }

    pub(crate) fn cached_entity_index(&self, entity: EntityId) -> Option<usize> {
        cached_query_entity_index(&self.cached_entity_indices, entity)
    }

    pub(crate) fn cached_entity_location(
        &self,
        entity: EntityId,
    ) -> Option<(StableEntityLocation, &[ComponentStorageLocation])> {
        let index = self.cached_entity_index(entity)?;
        let stable_location = self.cached_locations.get(index).copied()?;
        let component_locations = cached_query_component_locations(
            &self.cached_component_locations,
            &self.cached_component_location_offsets,
            index,
        )?;
        Some((stable_location, component_locations))
    }

    pub fn cached_location_count(&self) -> usize {
        self.cached_locations.len()
    }

    pub fn cached_locations(&self) -> &[StableEntityLocation] {
        &self.cached_locations
    }

    pub fn cached_component_locations(&self) -> &[ComponentStorageLocation] {
        &self.cached_component_locations
    }

    pub fn cached_component_location_offsets(&self) -> &[usize] {
        &self.cached_component_location_offsets
    }

    pub fn cached_revision(&self) -> u64 {
        self.cached_revision
    }

    pub fn cache_rebuilds(&self) -> u64 {
        self.cache_rebuilds
    }
}
