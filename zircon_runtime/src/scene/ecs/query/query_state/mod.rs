mod cache;
mod cached_direct;
mod many_item_array;
mod mutable;
mod read_only;
mod read_only_cached;
mod stats;
mod system_param;

use std::{cell::Cell, marker::PhantomData};

use crate::scene::ecs::{
    ArchetypeId, ChangeDetectionScanStats, ComponentStorageLocation, QueryAccess, QueryAccessError,
    QueryDataAccess, QueryFilter, StableEntityLocation,
};
use crate::scene::EntityId;
use crate::scene::World;

#[derive(Clone, Debug)]
pub struct QueryState<D, F = ()> {
    access: QueryAccess,
    cached_archetypes: Vec<ArchetypeId>,
    cached_archetype_generation: u64,
    cached_entities: Vec<EntityId>,
    cached_entity_indices: Vec<(EntityId, usize)>,
    cached_locations: Vec<StableEntityLocation>,
    cached_component_locations: Vec<ComponentStorageLocation>,
    cached_component_location_offsets: Vec<usize>,
    cached_revision: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_rebuilds: u64,
    last_reported_cache_stats: QueryStateCacheStats,
    change_detection_stats: Cell<ChangeDetectionScanStats>,
    last_reported_change_detection_stats: ChangeDetectionScanStats,
    last_candidate_entity_count: usize,
    last_matched_entity_count: usize,
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
            cached_component_location_offsets: Vec::new(),
            cached_revision: u64::MAX,
            cache_hits: 0,
            cache_misses: 0,
            cache_rebuilds: 0,
            last_reported_cache_stats: QueryStateCacheStats::default(),
            change_detection_stats: Cell::new(ChangeDetectionScanStats::default()),
            last_reported_change_detection_stats: ChangeDetectionScanStats::default(),
            last_candidate_entity_count: 0,
            last_matched_entity_count: 0,
            _marker: PhantomData,
        };
        state.update_cache(world);
        state.last_reported_cache_stats = state.cache_stats();
        Ok(state)
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn conflicts_with<OtherD, OtherF>(&self, other: &QueryState<OtherD, OtherF>) -> bool {
        self.access.conflicts_with(&other.access)
    }
}

pub use stats::{
    QueryStateCacheStats, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
};
