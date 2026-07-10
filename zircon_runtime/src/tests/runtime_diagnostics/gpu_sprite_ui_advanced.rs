use crate::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::support::{
    assert_render_bool_series, assert_render_byte_series, assert_render_count_series,
};

pub(super) fn assert_gpu_sprite_ui_advanced(snapshot: &RuntimeDiagnosticsSnapshot) {
    assert_render_count_series(
        &snapshot.store,
        "render.gpu_scene.primitive_count",
        5.0,
        &["gpu_scene"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.gpu_scene.instance_count",
        7.0,
        &["gpu_scene"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.gpu_scene.dirty_entry_count",
        3.0,
        &["gpu_scene", "upload"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.gpu_scene.uploaded_bytes",
        128.0,
        &["gpu_scene", "upload"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.gpu_scene.upload_path.direct_queue_write",
        true,
        &["gpu_scene", "upload", "direct_queue_write"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.gpu_scene.free_span_count",
        2.0,
        &["gpu_scene", "allocator"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.gpu_scene.primitive_upload_range_count",
        1.0,
        &["gpu_scene", "upload", "primitive"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.gpu_scene.instance_upload_range_count",
        4.0,
        &["gpu_scene", "upload", "instance"],
    );
    assert_render_count_series(&snapshot.store, "render.sprite.count", 11.0, &["sprite"]);
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.ready_count",
        9.0,
        &["sprite"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.texture_fallback_count",
        2.0,
        &["sprite", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.graph_executed_pass_count",
        3.0,
        &["sprite", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.draw_batch_count",
        4.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.batched_sprite_count",
        10.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.image_slice_count",
        14.0,
        &["sprite", "queue", "image_slice"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.expanded_image_slice_count",
        4.0,
        &["sprite", "queue", "image_slice", "expanded"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.vertex_count",
        60.0,
        &["sprite", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.opaque_draw_batch_count",
        1.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.alpha_mask_draw_batch_count",
        1.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.transparent_draw_batch_count",
        2.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(&snapshot.store, "render.ui.command_count", 17.0, &["ui"]);
    assert_render_count_series(&snapshot.store, "render.ui.quad_count", 8.0, &["ui"]);
    assert_render_count_series(
        &snapshot.store,
        "render.ui.text_payload_count",
        5.0,
        &["ui", "text"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.ui.image_payload_count",
        2.0,
        &["ui", "image"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.ui.clipped_command_count",
        3.0,
        &["ui", "clip"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.ui.graph_executed_pass_count",
        1.0,
        &["ui", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.cluster_budget",
        128.0,
        &["virtual_geometry", "budget"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.payload.source.authored",
        true,
        &["virtual_geometry", "payload", "source"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.debug.freeze_cull",
        true,
        &["virtual_geometry", "debug"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.resident_page_count",
        20.0,
        &["virtual_geometry", "page", "resident"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.execution_missing_segment_count",
        2.0,
        &["virtual_geometry", "execution", "missing"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.cluster_selection.input_source.prepare_on_demand",
        true,
        &["virtual_geometry", "cluster_selection", "source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.node_and_cluster_cull.dispatch_group_z",
        5.0,
        &["virtual_geometry", "cull", "dispatch"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.visbuffer64.source.render_path_execution_selections",
        true,
        &["virtual_geometry", "visbuffer64", "source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hybrid_gi.active_probe_count",
        5.0,
        &["hybrid_gi", "probe"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hybrid_gi.surface_cache.invalidated_page_count",
        15.0,
        &["hybrid_gi", "surface_cache", "invalidation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hybrid_gi.voxel.invalidated_clipmap_count",
        18.0,
        &["hybrid_gi", "voxel", "invalidation"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hybrid_gi.payload.source.scene_representation",
        true,
        &["hybrid_gi", "payload", "source"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.availability.virtual_geometry_provider_present",
        true,
        &["advanced_provider", "availability", "virtual_geometry"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.availability.hybrid_gi_provider_present",
        false,
        &["advanced_provider", "availability", "hybrid_gi"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.advanced_provider.report_count",
        2.0,
        &["advanced_provider"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.advanced_provider.enabled_count",
        1.0,
        &["advanced_provider", "enabled"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.virtual_geometry.ready",
        true,
        &["advanced_provider", "virtual_geometry", "ready"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.hybrid_gi.degraded",
        true,
        &["advanced_provider", "hybrid_gi", "degraded"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.advanced_provider.hybrid_gi.missing_provider_degradation_count",
        1.0,
        &["advanced_provider", "hybrid_gi", "degradation", "provider"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.solari.requested",
        true,
        &["solari", "requested"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.solari.enabled",
        false,
        &["solari", "enabled"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.solari.status.experimental_disabled",
        true,
        &["solari", "status", "experimental"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.solari.experimental_disabled_degradation_count",
        1.0,
        &["solari", "degradation", "experimental"],
    );
}
