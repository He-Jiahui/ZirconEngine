mod archetype_plan;
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
    ChangeDetectionScanStats, ComponentStorageLocation, QueryAccess, QueryAccessError,
    QueryDataAccess, QueryFilter, StableEntityLocation,
};
use crate::scene::EntityId;
use crate::scene::World;

pub(crate) use archetype_plan::{
    find_cached_archetype_plan, project_entity_from_plans, CachedArchetypePlan,
    QueryComponentBinding,
};

#[derive(Clone, Debug)]
pub struct QueryState<D, F = ()> {
    access: QueryAccess,
    cached_archetype_plans: Vec<CachedArchetypePlan>,
    cached_archetype_generation: u64,
    cached_entity_count: usize,
    cache_hits: u64,
    cache_misses: u64,
    cache_rebuilds: u64,
    archetype_plan_compilations: u64,
    archetype_component_membership_checks: u64,
    table_column_slot_bindings: u64,
    sparse_component_bindings: u64,
    archetype_index_component_probes: u64,
    archetype_index_signature_membership_checks: u64,
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
            cached_archetype_plans: Vec::new(),
            cached_archetype_generation: u64::MAX,
            cached_entity_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_rebuilds: 0,
            archetype_plan_compilations: 0,
            archetype_component_membership_checks: 0,
            table_column_slot_bindings: 0,
            sparse_component_bindings: 0,
            archetype_index_component_probes: 0,
            archetype_index_signature_membership_checks: 0,
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

    pub(crate) fn project_entity(
        &self,
        world: &World,
        entity: EntityId,
        component_locations: &mut Vec<ComponentStorageLocation>,
    ) -> Option<StableEntityLocation> {
        project_entity_from_plans(
            &self.cached_archetype_plans,
            world,
            entity,
            component_locations,
        )
    }
}

pub use stats::{
    QueryStateCacheStats, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
    ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC, ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC,
    ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC, ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC,
};
