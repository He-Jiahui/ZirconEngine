use crate::core::framework::render::RenderStats;

use super::{record_bool, record_bytes, record_count, DiagnosticStore};

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
        "render.post_process.graph.final_composite_present",
        frame_index,
        stats.last_post_process_final_composite_node.is_some(),
        &["render", "post_process", "graph", "final_composite"],
    );
}
