use crate::core::framework::render::RenderStats;

use super::super::{record_bytes, record_count, DiagnosticStore};
use super::{execution, execution_resources, materialization};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
    record_compiled_cache(store, frame_index, stats);
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
    execution::record_coverage(store, frame_index, stats);
    execution_resources::record(store, frame_index, stats);
    materialization::record(store, frame_index, stats);
    execution::record_aliases(store, frame_index, stats);
    execution::record_batches(store, frame_index, stats);
    execution::record_stage(store, frame_index, stats);
    execution::record_profile(store, frame_index, stats);
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
        "render.taa.reactive_mask_encoded_pass_count",
        frame_index,
        stats.last_taa_reactive_mask_encoded_pass_count,
        &["render", "taa", "reactive_mask"],
    );
    record_count(
        store,
        "render.taa.reactive_mask_encoded_write_bytes",
        frame_index,
        stats.last_taa_reactive_mask_encoded_write_bytes as usize,
        &["render", "taa", "reactive_mask"],
    );
    record_count(
        store,
        "render.taa.resolve_bind_group_create_count",
        frame_index,
        stats.last_taa_resolve_bind_group_create_count,
        &["render", "taa", "bind_group"],
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

fn record_compiled_cache(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
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
