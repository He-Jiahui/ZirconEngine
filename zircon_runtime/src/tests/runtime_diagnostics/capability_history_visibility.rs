use crate::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::support::{assert_render_bool_series, assert_render_count_series};

pub(super) fn assert_capability_history_visibility(snapshot: &RuntimeDiagnosticsSnapshot) {
    assert_render_count_series(
        &snapshot.store,
        "render.capability.queue_class_count",
        3.0,
        &["capability", "queue"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.surface_supported",
        true,
        &["capability", "surface"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.async_copy_supported",
        false,
        &["capability", "async_copy"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.storage_buffer_supported",
        true,
        &["capability", "storage_buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capability.max_storage_buffers_per_shader_stage",
        10.0,
        &["capability", "storage_buffer"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.buffer_readback_supported",
        false,
        &["capability", "readback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.inline_ray_query_supported",
        true,
        &["capability", "raytracing"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.texture_binding_array_supported",
        true,
        &["capability", "binding_array"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.partially_bound_binding_array_supported",
        false,
        &["capability", "binding_array"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.fxaa_supported",
        true,
        &["capability", "anti_alias"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.dlss_supported",
        false,
        &["capability", "anti_alias"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.neural_compute_supported",
        true,
        &["capability", "neural_compute"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.sparse_texture_supported",
        false,
        &["capability", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capability.max_msaa_samples",
        8.0,
        &["capability", "anti_alias"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.virtual_geometry_supported",
        true,
        &["capability", "virtual_geometry"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.hybrid_gi_supported",
        true,
        &["capability", "hybrid_gi"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.current_handle_present",
        true,
        &["history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.previous_handle_present",
        true,
        &["history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.previous_available",
        false,
        &["history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated",
        true,
        &["history", "invalidation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.target_width",
        1280.0,
        &["history", "target_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.target_height",
        720.0,
        &["history", "target_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.render_width",
        960.0,
        &["history", "render_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.render_height",
        540.0,
        &["history", "render_size"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.history_target_present",
        true,
        &["history", "copy"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.debug_marker_emitted",
        true,
        &["history", "copy", "debug_marker"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.copy.requested_count",
        5.0,
        &["history", "copy"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.copy.copied_count",
        5.0,
        &["history", "copy"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.copy.target_width",
        960.0,
        &["history", "copy", "target_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.copy.target_height",
        540.0,
        &["history", "copy", "target_size"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.scene_color_copied",
        true,
        &["history", "copy", "scene_color"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.global_illumination_copied",
        true,
        &["history", "copy", "global_illumination"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.ambient_occlusion_copied",
        true,
        &["history", "copy", "ambient_occlusion"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.screen_space_reflection_copied",
        false,
        &["history", "copy", "screen_space_reflection"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.hzb_furthest_copied",
        true,
        &["history", "copy", "hzb"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.exposure_copied",
        true,
        &["history", "copy", "exposure"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.copy.volumetric_scattering_copied",
        false,
        &["history", "copy", "volumetric_scattering"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.loop_submission_count",
        4.0,
        &["render", "camera", "execution"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.scheduled_count",
        3.0,
        &["render", "camera", "ordering"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.order_ambiguity_count",
        1.0,
        &["render", "camera", "ordering"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.view_count",
        2.0,
        &["render", "visibility"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.input_count",
        8.0,
        &["render", "visibility"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.layer_filtered_count",
        1.0,
        &["render", "visibility", "layer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.frustum_culled_count",
        3.0,
        &["render", "visibility", "frustum"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.occlusion_culled_count",
        18.0,
        &["render", "visibility", "occlusion"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.visible_count",
        3.0,
        &["render", "visibility", "visible"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.full_rebuild_count",
        0.0,
        &["render", "visibility", "static_index", "rebuild"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.incremental_update_count",
        1.0,
        &["render", "visibility", "static_index", "incremental"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.inserted_count",
        2.0,
        &["render", "visibility", "static_index", "change"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.updated_count",
        3.0,
        &["render", "visibility", "static_index", "change"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.removed_count",
        4.0,
        &["render", "visibility", "static_index", "change"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.indexed_entity_count",
        10.0,
        &["render", "visibility", "static_index", "entity"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.occupied_cell_count",
        7.0,
        &["render", "visibility", "static_index", "cell"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.visibility.static_index.main_view_prefilter_used",
        true,
        &[
            "render",
            "visibility",
            "static_index",
            "main_view",
            "prefilter",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.main_view_static_input_count",
        12.0,
        &["render", "visibility", "static_index", "main_view", "input"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.visibility.static_index.main_view_static_candidate_count",
        5.0,
        &[
            "render",
            "visibility",
            "static_index",
            "main_view",
            "candidate",
        ],
    );
}
