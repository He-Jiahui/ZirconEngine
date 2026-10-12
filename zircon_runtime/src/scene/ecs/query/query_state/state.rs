use std::{cell::Cell, marker::PhantomData, sync::Arc};

use crate::scene::ecs::{
    ChangeDetectionScanStats, ComponentStorageLocation, QueryAccess, QueryAccessError,
    QueryDataAccess, QueryFilter, StableEntityLocation,
};
use crate::scene::world::DetachPreparationOwner;
use crate::scene::{EntityId, World};

use super::{project_entity_from_plans, CachedArchetypePlan, QueryStateCacheStats};

#[derive(Clone, Debug)]
/// Keeps compiled access and cached plans paired with their original World.
/// Cached and mutable entry points require that identity; generic uncached
/// read-only helpers continue to fetch directly from their supplied World.
pub struct QueryState<D, F = ()> {
    pub(super) access: QueryAccess,
    pub(super) world_owner: DetachPreparationOwner,
    pub(super) cached_storage_owner: Arc<()>,
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
        Self::try_new(world).expect("query data access must be valid")
    }

    pub fn try_new(world: &mut World) -> Result<Self, QueryAccessError> {
        let mut access = QueryAccess::default();
        D::update_access(world, &mut access)?;
        F::update_access(world, &mut access)?;
        let mut state = Self {
            access,
            world_owner: world.owner_identity().clone(),
            cached_storage_owner: Arc::clone(world.query_storage_owner()),
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

    /// Tests exact World identity without exposing its private token.
    pub fn is_bound_to(&self, world: &World) -> bool {
        self.world_owner.is_same_world(world.owner_identity())
    }

    pub(super) fn assert_bound_to(&self, world: &World) {
        assert!(
            self.is_bound_to(world),
            "query state belongs to another World; rebind it before using cached plans"
        );
    }

    /// Rebuilds data/filter access and every cached plan for the supplied World.
    /// Ordinary access errors preserve the original query packet.
    pub fn rebind(&mut self, world: &mut World) -> Result<(), QueryAccessError> {
        let replacement = Self::try_new(world)?;
        *self = replacement;
        Ok(())
    }

    /// Returns original-owner metadata. Component IDs from different Worlds
    /// cannot be compared as if they described a single access domain.
    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn conflicts_with<OtherD, OtherF>(&self, other: &QueryState<OtherD, OtherF>) -> bool {
        assert!(
            self.world_owner.is_same_world(&other.world_owner),
            "query access conflicts require states from the same World"
        );
        self.access.conflicts_with(&other.access)
    }

    pub(crate) fn project_entity(
        &self,
        world: &World,
        entity: EntityId,
        component_locations: &mut Vec<ComponentStorageLocation>,
    ) -> Option<StableEntityLocation> {
        self.assert_bound_to(world);
        project_entity_from_plans(
            &self.cached_archetype_plans,
            world,
            entity,
            component_locations,
        )
    }
}
