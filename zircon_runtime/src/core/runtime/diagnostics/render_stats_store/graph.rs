mod execution_resources;

use crate::core::framework::render::RenderStats;

use super::{record_bool, record_bytes, record_count, record_microseconds, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_frame_graph(store, stats);
    record_post_process_graph(store, stats);
}

fn record_frame_graph(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.last_graph_executed_pass_count",
        frame_index,
        stats.last_graph_executed_pass_count,
        &["render", "graph"],
    );
    record_count(
        store,
        "render.graph.pass_count",
        frame_index,
        stats.last_graph_pass_count,
        &["render", "graph"],
    );
    record_count(
        store,
        "render.graph.culled_pass_count",
        frame_index,
        stats.last_graph_culled_pass_count,
        &["render", "graph", "culling"],
    );
    record_count(
        store,
        "render.graph.queue_fallback_pass_count",
        frame_index,
        stats.last_graph_queue_fallback_pass_count,
        &["render", "graph", "queue"],
    );
    record_count(
        store,
        "render.graph.resource_lifetime_count",
        frame_index,
        stats.last_graph_resource_lifetime_count,
        &["render", "graph", "resource"],
    );
    record_count(
        store,
        "render.graph.sparse_texture_lifetime_count",
        frame_index,
        stats.last_graph_sparse_texture_lifetime_count,
        &["render", "graph", "resource", "sparse_texture"],
    );
    record_count(
        store,
        "render.graph.planned_resource_access_count",
        frame_index,
        stats.last_graph_planned_resource_access_count,
        &["render", "graph", "resource"],
    );
    record_count(
        store,
        "render.graph.planned_dependency_count",
        frame_index,
        stats.last_graph_planned_dependency_count,
        &["render", "graph", "dependency"],
    );
    record_count(
        store,
        "render.graph.transient_texture_slot_count",
        frame_index,
        stats.last_graph_transient_texture_slot_count,
        &["render", "graph", "transient", "texture"],
    );
    record_count(
        store,
        "render.graph.sparse_texture_slot_count",
        frame_index,
        stats.last_graph_sparse_texture_slot_count,
        &["render", "graph", "transient", "texture", "sparse_texture"],
    );
    record_count(
        store,
        "render.graph.transient_buffer_slot_count",
        frame_index,
        stats.last_graph_transient_buffer_slot_count,
        &["render", "graph", "transient", "buffer"],
    );
    record_bytes(
        store,
        "render.graph.transient_texture_bytes_reserved",
        frame_index,
        stats.last_graph_transient_texture_bytes_reserved,
        &["render", "graph", "transient", "texture"],
    );
    record_bytes(
        store,
        "render.graph.transient_buffer_bytes_reserved",
        frame_index,
        stats.last_graph_transient_buffer_bytes_reserved,
        &["render", "graph", "transient", "buffer"],
    );
    record_bytes(
        store,
        "render.graph.transient_dense_bytes_reserved",
        frame_index,
        stats.last_graph_transient_dense_bytes_reserved,
        &["render", "graph", "transient"],
    );
    record_bytes(
        store,
        "render.graph.sparse_texture_virtual_bytes",
        frame_index,
        stats.last_graph_sparse_texture_virtual_bytes,
        &["render", "graph", "transient", "texture", "sparse_texture"],
    );
    record_graph_compiled_cache(store, frame_index, stats);
    record_count(
        store,
        "render.graph.executed_pass_count",
        frame_index,
        stats.last_graph_executed_pass_count,
        &["render", "graph"],
    );
    record_count(
        store,
        "render.graph.executed_resource_access_count",
        frame_index,
        stats.last_graph_executed_resource_access_count,
        &["render", "graph", "resource"],
    );
    record_count(
        store,
        "render.graph.executed_dependency_count",
        frame_index,
        stats.last_graph_executed_dependency_count,
        &["render", "graph", "dependency"],
    );
    record_graph_execution_coverage(store, frame_index, stats);
    execution_resources::record(store, frame_index, stats);
    record_graph_materialization(store, frame_index, stats);
    record_graph_execution_aliases(store, frame_index, stats);
    record_graph_stage_execution(store, frame_index, stats);
    record_graph_execution_profile(store, frame_index, stats);
    record_count(
        store,
        "render.graph.compute_dispatch_count",
        frame_index,
        stats.last_graph_compute_dispatch_count,
        &["render", "graph", "compute", "dispatch"],
    );
    record_count(
        store,
        "render.graph.compute_dispatch_group_count",
        frame_index,
        stats.last_graph_compute_dispatch_group_count,
        &["render", "graph", "compute", "dispatch"],
    );
    record_count(
        store,
        "render.graph.compute_storage_write_resource_count",
        frame_index,
        stats.last_graph_compute_storage_write_resource_count,
        &["render", "graph", "compute", "storage"],
    );
    record_count(
        store,
        "render.graph.compute_planned_workload_count",
        frame_index,
        stats.last_graph_compute_planned_workload_count,
        &["render", "graph", "compute", "workload"],
    );
    record_count(
        store,
        "render.graph.compute_matched_workload_count",
        frame_index,
        stats.last_graph_compute_matched_workload_count,
        &["render", "graph", "compute", "workload"],
    );
    record_count(
        store,
        "render.graph.compute_missing_dispatch_count",
        frame_index,
        stats.last_graph_compute_missing_dispatch_count,
        &["render", "graph", "compute", "workload"],
    );
    record_count(
        store,
        "render.graph.compute_workload_mismatch_count",
        frame_index,
        stats.last_graph_compute_workload_mismatch_count,
        &["render", "graph", "compute", "workload"],
    );
    record_count(
        store,
        "render.graph.compute_unexpected_dispatch_count",
        frame_index,
        stats.last_graph_compute_unexpected_dispatch_count,
        &["render", "graph", "compute", "workload"],
    );
    record_count(
        store,
        "render.graph.debug_marker_count",
        frame_index,
        stats.last_graph_executed_debug_markers.len(),
        &["render", "graph", "debug_marker"],
    );
    record_count(
        store,
        "render.graph.executed_anti_alias_pass_count",
        frame_index,
        stats.last_anti_alias_graph_executed_pass_count,
        &["render", "graph", "anti_alias"],
    );
    record_count(
        store,
        "render.graph.executed_virtual_geometry_pass_count",
        frame_index,
        stats.last_virtual_geometry_graph_executed_pass_count,
        &["render", "graph", "virtual_geometry"],
    );
    record_count(
        store,
        "render.graph.executed_hybrid_gi_pass_count",
        frame_index,
        stats.last_hybrid_gi_graph_executed_pass_count,
        &["render", "graph", "hybrid_gi"],
    );
    record_count(
        store,
        "render.graph.executed_particle_pass_count",
        frame_index,
        stats.last_particle_graph_executed_pass_count,
        &["render", "graph", "particle"],
    );
    record_count(
        store,
        "render.graph.executed_shadow_pass_count",
        frame_index,
        stats.last_shadow_graph_executed_pass_count,
        &["render", "graph", "shadow"],
    );
    record_count(
        store,
        "render.graph.executed_transparent_pass_count",
        frame_index,
        stats.last_transparent_graph_executed_pass_count,
        &["render", "graph", "transparent"],
    );
    record_count(
        store,
        "render.graph.executed_async_compute_pass_count",
        frame_index,
        stats.last_async_compute_pass_count,
        &["render", "graph", "async_compute"],
    );
}

fn record_graph_compiled_cache(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    record_count(
        store,
        "render.graph.compiled_cache.hit_count",
        frame_index,
        stats.last_graph_compiled_cache_hit_count,
        &["render", "graph", "compiled_cache", "hit"],
    );
    record_count(
        store,
        "render.graph.compiled_cache.miss_count",
        frame_index,
        stats.last_graph_compiled_cache_miss_count,
        &["render", "graph", "compiled_cache", "miss"],
    );
    record_count(
        store,
        "render.graph.compiled_cache.eviction_count",
        frame_index,
        stats.last_graph_compiled_cache_eviction_count,
        &["render", "graph", "compiled_cache", "eviction"],
    );
    record_count(
        store,
        "render.graph.compiled_cache.entry_count",
        frame_index,
        stats.last_graph_compiled_cache_entry_count,
        &["render", "graph", "compiled_cache"],
    );
}

fn record_graph_execution_coverage(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = stats.last_graph_execution_coverage_report;
    record_count(
        store,
        "render.graph.execution.coverage.planned_live_pass_count",
        frame_index,
        report.planned_live_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.executed_pass_count",
        frame_index,
        report.executed_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.matched_planned_pass_count",
        frame_index,
        report.matched_planned_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.missing_planned_pass_count",
        frame_index,
        report.missing_planned_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.unexpected_executed_pass_count",
        frame_index,
        report.unexpected_executed_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.duplicate_executed_pass_count",
        frame_index,
        report.duplicate_executed_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
}

fn record_graph_stage_execution(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = stats.last_graph_stage_execution_report;
    record_count(
        store,
        "render.graph.execution.stage.staged_pass_count",
        frame_index,
        report.staged_pass_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.unstaged_pass_count",
        frame_index,
        report.unstaged_pass_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.unique_stage_count",
        frame_index,
        report.unique_stage_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.transition_count",
        frame_index,
        report.stage_transition_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.order_violation_count",
        frame_index,
        report.stage_order_violation_count,
        &["render", "graph", "execution", "stage", "order"],
    );
}

fn record_graph_execution_aliases(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = &stats.last_graph_execution_alias_report;
    record_count(
        store,
        "render.graph.execution.alias.texture_logical_count",
        frame_index,
        report.texture_logical_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.texture_alias_count",
        frame_index,
        report.texture_alias_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.texture_backing_count",
        frame_index,
        report.texture_backing_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
            "backing",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.buffer_logical_count",
        frame_index,
        report.buffer_logical_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.buffer_alias_count",
        frame_index,
        report.buffer_alias_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.buffer_backing_count",
        frame_index,
        report.buffer_backing_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
            "backing",
        ],
    );
}

fn record_graph_materialization(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = stats.last_graph_materialization_report;
    record_count(
        store,
        "render.graph.materialization.required_resource_count",
        frame_index,
        report.required_resource_count(),
        &["render", "graph", "materialization", "resource"],
    );
    record_count(
        store,
        "render.graph.materialization.bound_resource_count",
        frame_index,
        report.bound_resource_count(),
        &["render", "graph", "materialization", "resource", "bound"],
    );
    record_count(
        store,
        "render.graph.materialization.missing_resource_count",
        frame_index,
        report.missing_resource_count(),
        &["render", "graph", "materialization", "resource", "missing"],
    );
    record_count(
        store,
        "render.graph.materialization.missing_materialized_resource_count",
        frame_index,
        report.missing_materialized_resource_count(),
        &[
            "render",
            "graph",
            "materialization",
            "resource",
            "missing",
            "typed",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.required_texture_count",
        frame_index,
        report.required_texture_count,
        &["render", "graph", "materialization", "texture"],
    );
    record_count(
        store,
        "render.graph.materialization.bound_texture_count",
        frame_index,
        report.bound_texture_count,
        &["render", "graph", "materialization", "texture", "bound"],
    );
    record_count(
        store,
        "render.graph.materialization.missing_texture_count",
        frame_index,
        report.missing_texture_count,
        &["render", "graph", "materialization", "texture", "missing"],
    );
    record_count(
        store,
        "render.graph.materialization.required_buffer_count",
        frame_index,
        report.required_buffer_count,
        &["render", "graph", "materialization", "buffer"],
    );
    record_count(
        store,
        "render.graph.materialization.bound_buffer_count",
        frame_index,
        report.bound_buffer_count,
        &["render", "graph", "materialization", "buffer", "bound"],
    );
    record_count(
        store,
        "render.graph.materialization.missing_buffer_count",
        frame_index,
        report.missing_buffer_count,
        &["render", "graph", "materialization", "buffer", "missing"],
    );
    record_count(
        store,
        "render.graph.materialization.required_external_count",
        frame_index,
        report.required_external_count,
        &["render", "graph", "materialization", "external"],
    );
    record_count(
        store,
        "render.graph.materialization.bound_external_count",
        frame_index,
        report.bound_external_count(),
        &["render", "graph", "materialization", "external", "bound"],
    );
    record_count(
        store,
        "render.graph.materialization.missing_external_count",
        frame_index,
        report.missing_external_count(),
        &["render", "graph", "materialization", "external", "missing"],
    );
    record_count(
        store,
        "render.graph.materialization.bound_required_external_count",
        frame_index,
        report.bound_required_external_count,
        &[
            "render",
            "graph",
            "materialization",
            "external",
            "required",
            "bound",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.missing_required_external_count",
        frame_index,
        report.missing_required_external_count,
        &[
            "render",
            "graph",
            "materialization",
            "external",
            "required",
            "missing",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.report_only_external_count",
        frame_index,
        report.report_only_external_count,
        &[
            "render",
            "graph",
            "materialization",
            "external",
            "report_only",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.bound_report_only_external_count",
        frame_index,
        report.bound_report_only_external_count,
        &[
            "render",
            "graph",
            "materialization",
            "external",
            "report_only",
            "bound",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.missing_report_only_external_count",
        frame_index,
        report.missing_report_only_external_count,
        &[
            "render",
            "graph",
            "materialization",
            "external",
            "report_only",
            "missing",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.stale_binding_count",
        frame_index,
        report.stale_binding_count(),
        &[
            "render",
            "graph",
            "materialization",
            "resource",
            "stale_binding",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.stale_texture_binding_count",
        frame_index,
        report.stale_texture_binding_count,
        &[
            "render",
            "graph",
            "materialization",
            "texture",
            "stale_binding",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.stale_buffer_binding_count",
        frame_index,
        report.stale_buffer_binding_count,
        &[
            "render",
            "graph",
            "materialization",
            "buffer",
            "stale_binding",
        ],
    );
    record_count(
        store,
        "render.graph.materialization.sparse_texture_reservation_count",
        frame_index,
        report.sparse_texture_reservation_count,
        &[
            "render",
            "graph",
            "materialization",
            "texture",
            "sparse_texture",
        ],
    );
}

fn record_graph_execution_profile(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = &stats.last_graph_execution_profile_report;
    record_count(
        store,
        "render.graph.execution.profile.pass_count",
        frame_index,
        report.pass_count(),
        &["render", "graph", "execution", "profile"],
    );
    record_microseconds(
        store,
        "render.graph.execution.profile.cpu_elapsed_total_us",
        frame_index,
        report.total_cpu_elapsed_micros(),
        &["render", "graph", "execution", "profile", "cpu"],
    );
    record_microseconds(
        store,
        "render.graph.execution.profile.cpu_elapsed_max_us",
        frame_index,
        report.max_cpu_elapsed_micros(),
        &["render", "graph", "execution", "profile", "cpu"],
    );
}

fn record_post_process_graph(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.post_process.graph.node_count",
        frame_index,
        stats.last_post_process_graph_node_count,
        &["render", "post_process", "graph"],
    );
    record_count(
        store,
        "render.post_process.graph.skipped_node_count",
        frame_index,
        stats.last_post_process_graph_skipped_node_count,
        &["render", "post_process", "graph"],
    );
    record_count(
        store,
        "render.post_process.graph.executed_node_count",
        frame_index,
        stats.last_post_process_graph_executed_nodes.len(),
        &["render", "post_process", "graph"],
    );
    record_bool(
        store,
        "render.post_process.graph.output_transfer_present",
        frame_index,
        stats.last_post_process_output_transfer_node.is_some(),
        &["render", "post_process", "graph", "output_transfer"],
    );
}
