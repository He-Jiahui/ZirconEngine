use crate::scene::ecs::{
    ArchetypeIndexPerformanceStats, ChangeDetectionScanStats, DetachedEntityBatchOperationStats,
    EcsFramePerformanceDiagnostics, NativeSystemCallbackTiming, QueryStateCacheStats,
};
use std::time::Duration;

use super::World;

impl World {
    pub fn reset_ecs_frame_performance_diagnostics(&mut self) {
        self.ecs_frame_performance_diagnostics = EcsFramePerformanceDiagnostics::default();
    }

    pub fn ecs_frame_performance_diagnostics(&self) -> EcsFramePerformanceDiagnostics {
        self.ecs_frame_performance_diagnostics
    }

    pub(crate) fn record_ecs_query_cache_stats(&mut self, stats: QueryStateCacheStats) {
        self.ecs_frame_performance_diagnostics
            .add_query_stats(stats);
    }

    pub(crate) fn record_ecs_archetype_index_stats(
        &mut self,
        stats: ArchetypeIndexPerformanceStats,
    ) {
        self.ecs_frame_performance_diagnostics
            .add_archetype_index_stats(stats);
    }

    pub(crate) fn record_ecs_change_detection_stats(&mut self, stats: ChangeDetectionScanStats) {
        self.ecs_frame_performance_diagnostics
            .add_change_detection_stats(stats);
    }

    pub(super) fn record_derived_state_hierarchy_validity(
        &mut self,
        snapshot_entities: usize,
        validated_entities: usize,
        parent_chain_steps: usize,
    ) {
        self.ecs_frame_performance_diagnostics
            .derived_state_mut()
            .record_hierarchy_validity(snapshot_entities, validated_entities, parent_chain_steps);
    }

    pub(super) fn record_derived_state_hierarchy_topology_rebuild(&mut self, entity_count: usize) {
        self.ecs_frame_performance_diagnostics
            .derived_state_mut()
            .record_hierarchy_topology_rebuild(entity_count);
    }

    pub(super) fn record_derived_state_active_propagation(&mut self, entity_count: usize) {
        self.ecs_frame_performance_diagnostics
            .derived_state_mut()
            .record_active_propagation(entity_count);
    }

    pub(super) fn record_derived_state_world_matrix_propagation(&mut self, entity_count: usize) {
        self.ecs_frame_performance_diagnostics
            .derived_state_mut()
            .record_world_matrix_propagation(entity_count);
    }

    pub(super) fn record_derived_state_node_cache_rebuild(&mut self, entity_count: usize) {
        self.ecs_frame_performance_diagnostics
            .derived_state_mut()
            .record_node_cache_rebuild(entity_count);
    }

    pub(crate) fn record_bundle_transaction_diagnostics(
        &mut self,
        final_archetype_transition: bool,
        archetype_assignments: u64,
        component_storage_moves: usize,
        lifecycle_events: usize,
        staged_value_allocations: usize,
    ) {
        self.ecs_frame_performance_diagnostics
            .bundle_transactions_mut()
            .record_commit(
                final_archetype_transition,
                archetype_assignments,
                component_storage_moves,
                lifecycle_events,
                staged_value_allocations,
            );
    }

    pub(crate) fn record_detached_entity_batch_commit(
        &mut self,
        stats: DetachedEntityBatchOperationStats,
    ) {
        self.ecs_frame_performance_diagnostics
            .detached_entity_batches_mut()
            .record_commit(stats);
    }

    pub(crate) fn record_detached_entity_batch_rejected_preflight(&mut self) {
        self.ecs_frame_performance_diagnostics
            .detached_entity_batches_mut()
            .record_rejected_preflight();
    }

    pub(crate) fn record_native_system_conflicts(&mut self, count: usize) {
        self.ecs_frame_performance_diagnostics
            .native_system_schedule_mut()
            .record_conflicts(count);
    }

    pub(crate) fn record_native_system_main_callback(
        &mut self,
        callback: Duration,
        conservative_world_writer: bool,
    ) {
        self.ecs_frame_performance_diagnostics
            .native_system_schedule_mut()
            .record_main_callback(callback, conservative_world_writer);
    }

    pub(crate) fn record_native_system_worker_batch(
        &mut self,
        timings: &[NativeSystemCallbackTiming],
        elapsed: Duration,
        scheduler_parallelism: usize,
        temporary_control_buffer_count: usize,
        temporary_control_buffer_bytes: usize,
    ) {
        self.ecs_frame_performance_diagnostics
            .native_system_schedule_mut()
            .record_worker_batch(
                timings,
                elapsed,
                scheduler_parallelism,
                temporary_control_buffer_count,
                temporary_control_buffer_bytes,
            );
    }
}
