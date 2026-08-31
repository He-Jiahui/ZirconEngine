use crate::core::CoreHandle;
use crate::core::diagnostics::{DiagnosticStore, FrameDiagnostics};

use super::{
    ArchetypeIndexPerformanceStats, BundleTransactionDiagnostics, ChangeDetectionScanStats,
    NativeSystemScheduleDiagnostics, QueryStateCacheStats,
};

pub const ECS_DETACHED_BATCH_COMMIT_COUNT_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.commit_count";
pub const ECS_DETACHED_BATCH_REJECTED_PREFLIGHTS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.rejected_preflights";
pub const ECS_DETACHED_BATCH_FULL_WORLD_CLONE_BYTES_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.full_world_clone_bytes";
pub const ECS_DETACHED_BATCH_NODE_RECORD_CLONE_BYTES_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.node_record_clone_bytes";
pub const ECS_DETACHED_BATCH_MOVED_ROWS_DIAGNOSTIC: &str = "scene.ecs.detached_batch.moved_rows";
pub const ECS_DETACHED_BATCH_MOVED_TABLE_COMPONENTS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.moved_table_components";
pub const ECS_DETACHED_BATCH_MOVED_SPARSE_COMPONENTS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.moved_sparse_components";
pub const ECS_DETACHED_BATCH_MOVED_DYNAMIC_COMPONENTS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.moved_dynamic_components";
pub const ECS_DETACHED_BATCH_SWAP_REPAIRS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.swap_repairs";
pub const ECS_DETACHED_BATCH_ARCHETYPE_PUBLICATIONS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.archetype_publications";
pub const ECS_DETACHED_BATCH_LIFECYCLE_EVENTS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.lifecycle_events";
pub const ECS_DETACHED_BATCH_GENERATION_ADVANCES_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.generation_advances";
pub const ECS_DETACHED_BATCH_ORDERED_REMOVALS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.ordered_removals";
pub const ECS_DETACHED_BATCH_HIERARCHY_INDEX_LOOKUPS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.hierarchy_index_lookups";
pub const ECS_DETACHED_BATCH_CAMERA_INDEX_LOOKUPS_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.camera_index_lookups";
pub const ECS_DETACHED_BATCH_ROLLBACK_BYTES_DIAGNOSTIC: &str =
    "scene.ecs.detached_batch.rollback_bytes";
pub const ECS_DERIVED_STATE_HIERARCHY_VALIDITY_PASSES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.hierarchy_validity_passes";
pub const ECS_DERIVED_STATE_HIERARCHY_PARENT_SNAPSHOT_ENTITIES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.hierarchy_parent_snapshot_entities";
pub const ECS_DERIVED_STATE_HIERARCHY_VALIDITY_ENTITIES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.hierarchy_validity_entities";
pub const ECS_DERIVED_STATE_HIERARCHY_PARENT_CHAIN_STEPS_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.hierarchy_parent_chain_steps";
pub const ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILDS_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.hierarchy_topology_rebuilds";
pub const ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILD_ENTITIES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.hierarchy_topology_rebuild_entities";
pub const ECS_DERIVED_STATE_ACTIVE_PROPAGATION_PASSES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.active_propagation_passes";
pub const ECS_DERIVED_STATE_ACTIVE_PROPAGATION_ENTITIES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.active_propagation_entities";
pub const ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_PASSES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.world_matrix_propagation_passes";
pub const ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_ENTITIES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.world_matrix_propagation_entities";
pub const ECS_DERIVED_STATE_NODE_CACHE_REBUILDS_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.node_cache_rebuilds";
pub const ECS_DERIVED_STATE_NODE_CACHE_REBUILT_ENTITIES_DIAGNOSTIC: &str =
    "scene.ecs.derived_state.node_cache_rebuilt_entities";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DetachedEntityBatchOperationStats {
    pub(crate) moved_rows: u64,
    pub(crate) moved_table_components: u64,
    pub(crate) moved_sparse_components: u64,
    pub(crate) moved_dynamic_components: u64,
    pub(crate) swap_repairs: u64,
    pub(crate) archetype_publications: u64,
    pub(crate) lifecycle_events: u64,
    pub(crate) ordered_removals: u64,
    pub(crate) hierarchy_index_lookups: u64,
    pub(crate) camera_index_lookups: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DetachedEntityBatchDiagnostics {
    pub commit_count: u64,
    pub rejected_preflights: u64,
    pub full_world_clone_bytes: u64,
    pub node_record_clone_bytes: u64,
    pub moved_rows: u64,
    pub moved_table_components: u64,
    pub moved_sparse_components: u64,
    pub moved_dynamic_components: u64,
    pub swap_repairs: u64,
    pub archetype_publications: u64,
    pub lifecycle_events: u64,
    pub generation_advances: u64,
    pub ordered_removals: u64,
    pub hierarchy_index_lookups: u64,
    pub camera_index_lookups: u64,
    pub rollback_bytes: u64,
}

impl DetachedEntityBatchDiagnostics {
    pub(crate) fn record_commit(&mut self, stats: DetachedEntityBatchOperationStats) {
        self.commit_count = self.commit_count.saturating_add(1);
        self.moved_rows = self.moved_rows.saturating_add(stats.moved_rows);
        self.moved_table_components = self
            .moved_table_components
            .saturating_add(stats.moved_table_components);
        self.moved_sparse_components = self
            .moved_sparse_components
            .saturating_add(stats.moved_sparse_components);
        self.moved_dynamic_components = self
            .moved_dynamic_components
            .saturating_add(stats.moved_dynamic_components);
        self.swap_repairs = self.swap_repairs.saturating_add(stats.swap_repairs);
        self.archetype_publications = self
            .archetype_publications
            .saturating_add(stats.archetype_publications);
        self.lifecycle_events = self.lifecycle_events.saturating_add(stats.lifecycle_events);
        self.generation_advances = self.generation_advances.saturating_add(1);
        self.ordered_removals = self.ordered_removals.saturating_add(stats.ordered_removals);
        self.hierarchy_index_lookups = self
            .hierarchy_index_lookups
            .saturating_add(stats.hierarchy_index_lookups);
        self.camera_index_lookups = self
            .camera_index_lookups
            .saturating_add(stats.camera_index_lookups);
    }

    pub(crate) fn record_rejected_preflight(&mut self) {
        self.rejected_preflights = self.rejected_preflights.saturating_add(1);
    }

    fn diagnostic_values(&self) -> [(&'static str, f64); 16] {
        [
            (
                ECS_DETACHED_BATCH_COMMIT_COUNT_DIAGNOSTIC,
                self.commit_count as f64,
            ),
            (
                ECS_DETACHED_BATCH_REJECTED_PREFLIGHTS_DIAGNOSTIC,
                self.rejected_preflights as f64,
            ),
            (
                ECS_DETACHED_BATCH_FULL_WORLD_CLONE_BYTES_DIAGNOSTIC,
                self.full_world_clone_bytes as f64,
            ),
            (
                ECS_DETACHED_BATCH_NODE_RECORD_CLONE_BYTES_DIAGNOSTIC,
                self.node_record_clone_bytes as f64,
            ),
            (
                ECS_DETACHED_BATCH_MOVED_ROWS_DIAGNOSTIC,
                self.moved_rows as f64,
            ),
            (
                ECS_DETACHED_BATCH_MOVED_TABLE_COMPONENTS_DIAGNOSTIC,
                self.moved_table_components as f64,
            ),
            (
                ECS_DETACHED_BATCH_MOVED_SPARSE_COMPONENTS_DIAGNOSTIC,
                self.moved_sparse_components as f64,
            ),
            (
                ECS_DETACHED_BATCH_MOVED_DYNAMIC_COMPONENTS_DIAGNOSTIC,
                self.moved_dynamic_components as f64,
            ),
            (
                ECS_DETACHED_BATCH_SWAP_REPAIRS_DIAGNOSTIC,
                self.swap_repairs as f64,
            ),
            (
                ECS_DETACHED_BATCH_ARCHETYPE_PUBLICATIONS_DIAGNOSTIC,
                self.archetype_publications as f64,
            ),
            (
                ECS_DETACHED_BATCH_LIFECYCLE_EVENTS_DIAGNOSTIC,
                self.lifecycle_events as f64,
            ),
            (
                ECS_DETACHED_BATCH_GENERATION_ADVANCES_DIAGNOSTIC,
                self.generation_advances as f64,
            ),
            (
                ECS_DETACHED_BATCH_ORDERED_REMOVALS_DIAGNOSTIC,
                self.ordered_removals as f64,
            ),
            (
                ECS_DETACHED_BATCH_HIERARCHY_INDEX_LOOKUPS_DIAGNOSTIC,
                self.hierarchy_index_lookups as f64,
            ),
            (
                ECS_DETACHED_BATCH_CAMERA_INDEX_LOOKUPS_DIAGNOSTIC,
                self.camera_index_lookups as f64,
            ),
            (
                ECS_DETACHED_BATCH_ROLLBACK_BYTES_DIAGNOSTIC,
                self.rollback_bytes as f64,
            ),
        ]
    }
}

/// Deterministic work counters for scene derived-state maintenance.
///
/// These counters intentionally describe rows and parent-chain steps instead
/// of elapsed time, so structural regressions remain observable across hosts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldDerivedStateDiagnostics {
    pub hierarchy_validity_passes: u64,
    pub hierarchy_parent_snapshot_entities: u64,
    pub hierarchy_validity_entities: u64,
    pub hierarchy_parent_chain_steps: u64,
    pub hierarchy_topology_rebuilds: u64,
    pub hierarchy_topology_rebuild_entities: u64,
    pub active_propagation_passes: u64,
    pub active_propagation_entities: u64,
    pub world_matrix_propagation_passes: u64,
    pub world_matrix_propagation_entities: u64,
    pub node_cache_rebuilds: u64,
    pub node_cache_rebuilt_entities: u64,
}

impl WorldDerivedStateDiagnostics {
    pub(crate) fn record_hierarchy_validity(
        &mut self,
        snapshot_entities: usize,
        validated_entities: usize,
        parent_chain_steps: usize,
    ) {
        self.hierarchy_validity_passes = self.hierarchy_validity_passes.saturating_add(1);
        self.hierarchy_parent_snapshot_entities = self
            .hierarchy_parent_snapshot_entities
            .saturating_add(snapshot_entities as u64);
        self.hierarchy_validity_entities = self
            .hierarchy_validity_entities
            .saturating_add(validated_entities as u64);
        self.hierarchy_parent_chain_steps = self
            .hierarchy_parent_chain_steps
            .saturating_add(parent_chain_steps as u64);
    }

    pub(crate) fn record_hierarchy_topology_rebuild(&mut self, entity_count: usize) {
        self.hierarchy_topology_rebuilds = self.hierarchy_topology_rebuilds.saturating_add(1);
        self.hierarchy_topology_rebuild_entities = self
            .hierarchy_topology_rebuild_entities
            .saturating_add(entity_count as u64);
    }

    pub(crate) fn record_active_propagation(&mut self, entity_count: usize) {
        self.active_propagation_passes = self.active_propagation_passes.saturating_add(1);
        self.active_propagation_entities = self
            .active_propagation_entities
            .saturating_add(entity_count as u64);
    }

    pub(crate) fn record_world_matrix_propagation(&mut self, entity_count: usize) {
        self.world_matrix_propagation_passes =
            self.world_matrix_propagation_passes.saturating_add(1);
        self.world_matrix_propagation_entities = self
            .world_matrix_propagation_entities
            .saturating_add(entity_count as u64);
    }

    pub(crate) fn record_node_cache_rebuild(&mut self, entity_count: usize) {
        self.node_cache_rebuilds = self.node_cache_rebuilds.saturating_add(1);
        self.node_cache_rebuilt_entities = self
            .node_cache_rebuilt_entities
            .saturating_add(entity_count as u64);
    }

    fn diagnostic_values(&self) -> [(&'static str, f64); 12] {
        [
            (
                ECS_DERIVED_STATE_HIERARCHY_VALIDITY_PASSES_DIAGNOSTIC,
                self.hierarchy_validity_passes as f64,
            ),
            (
                ECS_DERIVED_STATE_HIERARCHY_PARENT_SNAPSHOT_ENTITIES_DIAGNOSTIC,
                self.hierarchy_parent_snapshot_entities as f64,
            ),
            (
                ECS_DERIVED_STATE_HIERARCHY_VALIDITY_ENTITIES_DIAGNOSTIC,
                self.hierarchy_validity_entities as f64,
            ),
            (
                ECS_DERIVED_STATE_HIERARCHY_PARENT_CHAIN_STEPS_DIAGNOSTIC,
                self.hierarchy_parent_chain_steps as f64,
            ),
            (
                ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILDS_DIAGNOSTIC,
                self.hierarchy_topology_rebuilds as f64,
            ),
            (
                ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILD_ENTITIES_DIAGNOSTIC,
                self.hierarchy_topology_rebuild_entities as f64,
            ),
            (
                ECS_DERIVED_STATE_ACTIVE_PROPAGATION_PASSES_DIAGNOSTIC,
                self.active_propagation_passes as f64,
            ),
            (
                ECS_DERIVED_STATE_ACTIVE_PROPAGATION_ENTITIES_DIAGNOSTIC,
                self.active_propagation_entities as f64,
            ),
            (
                ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_PASSES_DIAGNOSTIC,
                self.world_matrix_propagation_passes as f64,
            ),
            (
                ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_ENTITIES_DIAGNOSTIC,
                self.world_matrix_propagation_entities as f64,
            ),
            (
                ECS_DERIVED_STATE_NODE_CACHE_REBUILDS_DIAGNOSTIC,
                self.node_cache_rebuilds as f64,
            ),
            (
                ECS_DERIVED_STATE_NODE_CACHE_REBUILT_ENTITIES_DIAGNOSTIC,
                self.node_cache_rebuilt_entities as f64,
            ),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EcsFramePerformanceDiagnostics {
    pub bundle_transactions: BundleTransactionDiagnostics,
    pub detached_entity_batches: DetachedEntityBatchDiagnostics,
    pub archetype_index: ArchetypeIndexPerformanceStats,
    pub query: QueryStateCacheStats,
    pub change_detection: ChangeDetectionScanStats,
    pub native_system_schedule: NativeSystemScheduleDiagnostics,
    pub derived_state: WorldDerivedStateDiagnostics,
}

impl EcsFramePerformanceDiagnostics {
    pub fn new(query: QueryStateCacheStats, change_detection: ChangeDetectionScanStats) -> Self {
        let archetype_index = ArchetypeIndexPerformanceStats {
            component_index_probes: query.archetype_index_component_probes,
            signature_membership_checks: query.archetype_index_signature_membership_checks,
            ..Default::default()
        };
        Self {
            bundle_transactions: BundleTransactionDiagnostics::default(),
            detached_entity_batches: DetachedEntityBatchDiagnostics::default(),
            archetype_index,
            query,
            change_detection,
            native_system_schedule: NativeSystemScheduleDiagnostics::default(),
            derived_state: WorldDerivedStateDiagnostics::default(),
        }
    }

    pub fn add_query_stats(&mut self, stats: QueryStateCacheStats) {
        self.query.cache_hits = self.query.cache_hits.saturating_add(stats.cache_hits);
        self.query.cache_misses = self.query.cache_misses.saturating_add(stats.cache_misses);
        self.query.cache_rebuilds = self
            .query
            .cache_rebuilds
            .saturating_add(stats.cache_rebuilds);
        self.query.archetype_plan_compilations = self
            .query
            .archetype_plan_compilations
            .saturating_add(stats.archetype_plan_compilations);
        self.query.archetype_component_membership_checks = self
            .query
            .archetype_component_membership_checks
            .saturating_add(stats.archetype_component_membership_checks);
        self.query.table_column_slot_bindings = self
            .query
            .table_column_slot_bindings
            .saturating_add(stats.table_column_slot_bindings);
        self.query.sparse_component_bindings = self
            .query
            .sparse_component_bindings
            .saturating_add(stats.sparse_component_bindings);
        self.archetype_index.component_index_probes = self
            .archetype_index
            .component_index_probes
            .saturating_add(stats.archetype_index_component_probes);
        self.archetype_index.signature_membership_checks = self
            .archetype_index
            .signature_membership_checks
            .saturating_add(stats.archetype_index_signature_membership_checks);
        self.query.cached_revision = self.query.cached_revision.max(stats.cached_revision);
        self.query.cached_archetype_count = self
            .query
            .cached_archetype_count
            .saturating_add(stats.cached_archetype_count);
        self.query.cached_entity_count = self
            .query
            .cached_entity_count
            .saturating_add(stats.cached_entity_count);
        self.query.candidate_entity_count = self
            .query
            .candidate_entity_count
            .saturating_add(stats.candidate_entity_count);
        self.query.matched_entity_count = self
            .query
            .matched_entity_count
            .saturating_add(stats.matched_entity_count);
    }

    pub fn add_archetype_index_stats(&mut self, stats: ArchetypeIndexPerformanceStats) {
        self.archetype_index.component_index_probes = self
            .archetype_index
            .component_index_probes
            .saturating_add(stats.component_index_probes);
        self.archetype_index.signature_membership_checks = self
            .archetype_index
            .signature_membership_checks
            .saturating_add(stats.signature_membership_checks);
        self.archetype_index.row_appends = self
            .archetype_index
            .row_appends
            .saturating_add(stats.row_appends);
    }

    pub fn add_change_detection_stats(&mut self, stats: ChangeDetectionScanStats) {
        self.change_detection.merge(stats);
    }

    pub(crate) fn bundle_transactions_mut(&mut self) -> &mut BundleTransactionDiagnostics {
        &mut self.bundle_transactions
    }

    pub(crate) fn detached_entity_batches_mut(&mut self) -> &mut DetachedEntityBatchDiagnostics {
        &mut self.detached_entity_batches
    }

    pub(crate) fn native_system_schedule_mut(&mut self) -> &mut NativeSystemScheduleDiagnostics {
        &mut self.native_system_schedule
    }

    pub(crate) fn derived_state_mut(&mut self) -> &mut WorldDerivedStateDiagnostics {
        &mut self.derived_state
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.bundle_transactions
            .record_diagnostics(store, frame_index);
        for (path, value) in self.detached_entity_batches.diagnostic_values() {
            store.record(
                path,
                frame_index,
                value,
                Some("count"),
                ["ecs", "detached_batch"],
            );
        }
        for (path, value) in self.archetype_index.diagnostic_values() {
            store.record(
                path,
                frame_index,
                value,
                Some("count"),
                ["ecs", "archetype"],
            );
        }
        self.query.record_diagnostics(store, frame_index);
        self.change_detection.record_diagnostics(store, frame_index);
        for (path, value) in self.derived_state.diagnostic_values() {
            store.record(
                path,
                frame_index,
                value,
                Some("count"),
                ["ecs", "derived_state"],
            );
        }
        self.native_system_schedule
            .record_diagnostics(store, frame_index);
    }

    pub fn publish(&self, core: &CoreHandle, frame_index: u64) {
        core.update_diagnostic_store(|store| self.record_diagnostics(store, frame_index));
    }
}

impl FrameDiagnostics for EcsFramePerformanceDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "scene.ecs"
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::CoreRuntime;
    use crate::core::diagnostics::{DiagnosticStore, FrameDiagnostics};

    use super::EcsFramePerformanceDiagnostics;

    #[test]
    fn ecs_frame_performance_diagnostics_uses_scene_ecs_frame_domain() {
        let diagnostics = EcsFramePerformanceDiagnostics::default();
        let status = diagnostics.frame_diagnostics_status();

        assert_eq!(status.domain, "scene.ecs");
        assert!(status.available);
        assert_eq!(status.error, None);
    }

    #[test]
    fn optimization_wave_20260824p_runtime03_ecs_publish_matches_direct_store_recording() {
        let diagnostics = EcsFramePerformanceDiagnostics::default();
        let mut expected = DiagnosticStore::default();
        diagnostics.record_diagnostics(&mut expected, 41);

        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        diagnostics.publish(&core, 41);

        let actual = core.diagnostic_store_snapshot();
        assert_eq!(actual, expected.snapshot());
        assert_eq!(actual.series.len(), 58);
    }

    #[test]
    fn optimization_wave_20260824p_runtime03_ecs_publish_uses_one_diagnostic_store_update() {
        let source = include_str!("frame_performance_diagnostics.rs");
        let publish = source
            .split("pub fn publish(&self, core: &CoreHandle, frame_index: u64)")
            .nth(1)
            .and_then(|source| source.split("impl FrameDiagnostics").next())
            .expect("ECS diagnostics publish implementation");

        assert_eq!(publish.matches("update_diagnostic_store").count(), 1);
        assert!(!publish.contains(".record_diagnostic("));
        assert!(!publish.contains("bundle_transactions.publish"));
        assert!(!publish.contains("native_system_schedule.publish"));
    }

    #[test]
    #[ignore = "managed Runtime03 performance evidence"]
    fn optimization_wave_20260824p_runtime03_ecs_diagnostic_batch_publish_evidence() {
        const SERIES_PER_PUBLISH: u64 = 58;
        const PUBLISHES: u64 = 25_000;
        const MAX_ELAPSED: Duration = Duration::from_secs(3);

        let diagnostics = EcsFramePerformanceDiagnostics::default();
        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        let started = Instant::now();
        for frame_index in 0..PUBLISHES {
            black_box(&diagnostics).publish(black_box(&core), frame_index);
        }
        let elapsed = started.elapsed();

        assert!(elapsed <= MAX_ELAPSED);
        let lock_acquisitions_before = SERIES_PER_PUBLISH * PUBLISHES;
        let lock_acquisitions_after = PUBLISHES;
        let lock_reduction_percent =
            (1.0 - lock_acquisitions_after as f64 / lock_acquisitions_before as f64) * 100.0;
        let publishes_per_second = PUBLISHES as f64 / elapsed.as_secs_f64();
        println!(
            "RUNTIME_DIAGNOSTICS_BENCH_V1 kind=ecs_batch_publish publishes={} series_per_publish={} lock_acquisitions_before={} lock_acquisitions_after={} lock_reduction_percent={:.4} elapsed_ns={} target_ns={} publishes_per_second={:.2}",
            PUBLISHES,
            SERIES_PER_PUBLISH,
            lock_acquisitions_before,
            lock_acquisitions_after,
            lock_reduction_percent,
            elapsed.as_nanos(),
            MAX_ELAPSED.as_nanos(),
            publishes_per_second,
        );
    }
}
