use std::{cell::Cell, marker::PhantomData};

use crate::scene::ecs::{
    ChangeDetectionScanStats, ComponentStorageLocation, QueryAccess, QueryAccessError,
    QueryDataAccess, QueryFilter, StableEntityLocation,
};
use crate::scene::{EntityId, World};

use super::{CachedArchetypePlan, QueryStateCacheStats, project_entity_from_plans};

#[derive(Clone, Debug)]
pub struct QueryState<D, F = ()> {
    pub(super) access: QueryAccess,
    pub(super) cached_archetype_plans: Vec<CachedArchetypePlan>,
    pub(super) cached_archetype_generation: u64,
    pub(super) cached_entity_count: usize,
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) cache_rebuilds: u64,
    pub(super) archetype_plan_compilations: u64,
    pub(super) archetype_component_membership_checks: u64,
    pub(super) table_column_slot_bindings: u64,
    pub(super) sparse_component_bindings: u64,
    pub(super) archetype_index_component_probes: u64,
    pub(super) archetype_index_signature_membership_checks: u64,
    pub(super) last_reported_cache_stats: QueryStateCacheStats,
    pub(super) change_detection_stats: Cell<ChangeDetectionScanStats>,
    pub(super) last_reported_change_detection_stats: ChangeDetectionScanStats,
    pub(super) last_candidate_entity_count: usize,
    pub(super) last_matched_entity_count: usize,
    pub(super) _marker: PhantomData<fn() -> (D, F)>,
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
