use super::super::sources::HotspotInventorySources;

pub(super) fn assert_ecs_frame_diagnostic_aggregation(sources: &HotspotInventorySources) {
    for required_schedule_span_anchor in [
        "profile_scope!",
        "\"runtime\"",
        "\"frame\"",
        "schedule_stage_profile_name(stage)",
        "runtime_frame_schedule_stage.RenderExtract",
    ] {
        assert!(
            sources
                .schedule_runner
                .contains(required_schedule_span_anchor),
            "SceneScheduleRunner should keep Runtime 07 schedule-stage span anchor `{required_schedule_span_anchor}`"
        );
    }
    assert!(
        !sources
            .schedule_runner
            .contains("format!(\"runtime_frame_schedule_stage"),
        "SceneScheduleRunner stage spans should use static labels without per-frame formatting"
    );

    for required_ecs_frame_diagnostic_anchor in [
        "pub struct EcsFramePerformanceDiagnostics",
        "pub fn add_query_stats(&mut self, stats: QueryStateCacheStats)",
        "pub fn add_change_detection_stats(&mut self, stats: ChangeDetectionScanStats)",
        "pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64)",
        "pub fn publish(&self, core: &CoreHandle, frame_index: u64)",
        "ecs_frame_performance_diagnostics_record_query_and_change_counts",
        "last_reported_cache_stats",
        "change_detection_stats: Cell<ChangeDetectionScanStats>",
        "last_reported_change_detection_stats",
        "pub fn saturating_delta_since(self, baseline: Self) -> Self",
        "pub(crate) fn take_unreported_cache_stats(&mut self) -> QueryStateCacheStats",
        "pub(crate) fn take_unreported_change_detection_stats(&mut self) -> ChangeDetectionScanStats",
        "fn record_performance_diagnostics(world: &mut World, state: &mut Self::State)",
        "world.record_ecs_query_cache_stats(query_stats);",
        "world.record_ecs_change_detection_stats(change_detection_stats);",
        "state.record_change_detection_stats(self.change_detection_stats);",
        "state: Option<NonNull<QueryState<D, F>>>",
        "P::record_performance_diagnostics(world, &mut self.state);",
        "P::record_performance_diagnostics(world, state);",
        "A::record_performance_diagnostics(world, &mut state.0);",
        "pub fn reset_ecs_frame_performance_diagnostics(&mut self)",
        "pub fn ecs_frame_performance_diagnostics(&self) -> EcsFramePerformanceDiagnostics",
        "pub(crate) fn record_ecs_query_cache_stats(&mut self, stats: QueryStateCacheStats)",
        "pub(crate) fn record_ecs_change_detection_stats(&mut self, stats: ChangeDetectionScanStats)",
        "level.with_world_mut(|world| world.reset_ecs_frame_performance_diagnostics());",
        ".ecs_frame_performance_diagnostics()",
        ".publish(core, core.real_time().frame_index());",
        "system_state_records_query_cache_stats_into_world_frame_diagnostics",
        "system_state_records_change_detection_stats_into_world_frame_diagnostics",
        "matches_component_locations_with_stats",
        "stats.scan_changed(component_ticks, ticks)",
        "stats.scan_added(component_ticks, ticks)",
    ] {
        assert!(
            sources
                .ecs_frame_diagnostics
                .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_filter
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_iter
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_many_iter
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_state_state
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_state_cache
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_state_read_only_cached
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_state_stats
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_state_system_param
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .system_param
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .system_state
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .param_set
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .world_performance_diagnostics
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .world_driver
                    .contains(required_ecs_frame_diagnostic_anchor)
                || sources
                    .query_tests
                    .contains(required_ecs_frame_diagnostic_anchor),
            "ECS frame performance diagnostic aggregation should retain `{required_ecs_frame_diagnostic_anchor}`"
        );
    }
}
