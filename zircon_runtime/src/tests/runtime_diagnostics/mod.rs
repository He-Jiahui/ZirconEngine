mod motion_vector;
mod support;

use crate::core::CoreRuntime;

use support::{
    assert_light_family_series, assert_render_bool_series, assert_render_byte_series,
    assert_render_count_series, assert_series_current, fake_render_module, DIAGNOSTICS_TEST_MODULE,
};

#[test]
fn runtime_diagnostics_reports_missing_runtime_contracts_without_panicking() {
    let runtime = CoreRuntime::new();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(!snapshot.render.available);
    assert!(snapshot.render.stats.is_none());
    assert!(snapshot.render.error.is_some());
    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());
    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());
    assert!(snapshot.store.is_empty());
}

#[test]
fn runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins() {
    let runtime = CoreRuntime::new();
    runtime.register_module(fake_render_module()).unwrap();
    runtime.activate_module(DIAGNOSTICS_TEST_MODULE).unwrap();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(snapshot.render.available);
    let render_stats = snapshot.render.stats.expect("render stats");
    assert_eq!(render_stats.active_viewports, 2);
    assert_eq!(render_stats.submitted_frames, 7);
    assert_eq!(
        render_stats.capabilities.backend_name,
        "diagnostics-test-renderer"
    );
    assert!(!snapshot.render.virtual_geometry_debug_available);
    assert!(snapshot.render.error.is_none());

    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());

    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());

    assert!(snapshot
        .store
        .series
        .iter()
        .any(|series| series.path.as_str() == "render.submitted_frames"
            && series.current == Some(7.0)));
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
        4.0,
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
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.mip_count",
        10.0,
        &["render", "hzb", "mip"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.graph_executed_pass_count",
        1.0,
        &["render", "hzb", "graph"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.reported",
        true,
        &["render", "hzb", "occlusion"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.candidate_arg_count",
        6.0,
        &["render", "hzb", "occlusion", "candidate"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.candidate_instance_count",
        42.0,
        &["render", "hzb", "occlusion", "candidate"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.dispatch_group_count",
        2.0,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.dispatched_phase_count",
        1.0,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.history_available",
        true,
        &["render", "hzb", "occlusion", "history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.readback_available",
        true,
        &["render", "hzb", "occlusion", "readback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.tested_arg_count",
        6.0,
        &["render", "hzb", "occlusion", "tested"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.tested_instance_count",
        42.0,
        &["render", "hzb", "occlusion", "tested"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.culled_arg_count",
        2.0,
        &["render", "hzb", "occlusion", "culled"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.culled_instance_count",
        18.0,
        &["render", "hzb", "occlusion", "culled"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.indirect_args_readback_available",
        true,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.readback_arg_count",
        6.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.compacted_draw_count",
        4.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.zero_instance_arg_count",
        2.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.remaining_instance_count",
        24.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.light_grid.reported",
        true,
        &["render", "light_grid"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.light_count",
        9.0,
        &["render", "light_grid", "light"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.tile_count",
        64.0,
        &["render", "light_grid", "tile"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.zbin_count",
        32.0,
        &["render", "light_grid", "zbin"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.non_empty_tile_count",
        11.0,
        &["render", "light_grid", "tile"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.non_empty_zbin_count",
        7.0,
        &["render", "light_grid", "zbin"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.non_empty_cluster_count",
        23.0,
        &["render", "light_grid", "cluster"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.peak_lights_per_cluster",
        5.0,
        &["render", "light_grid", "cluster", "peak"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.average_lights_per_cluster",
        0.375,
        &["render", "light_grid", "cluster", "average"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.primary_surface",
        false,
        &["render", "camera", "target", "primary_surface"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.headless",
        true,
        &["render", "camera", "target", "headless"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.texture",
        false,
        &["render", "camera", "target", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.primary_width",
        1280.0,
        &["render", "camera", "target", "primary"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.primary_height",
        720.0,
        &["render", "camera", "target", "primary"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.resolved_width",
        640.0,
        &["render", "camera", "target", "resolved"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.resolved_height",
        360.0,
        &["render", "camera", "target", "resolved"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_view_width",
        320.0,
        &["render", "camera", "target", "effective_view"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_view_height",
        180.0,
        &["render", "camera", "target", "effective_view"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_render_width",
        160.0,
        &["render", "camera", "target", "effective_render"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_render_height",
        90.0,
        &["render", "camera", "target", "effective_render"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.not_requested",
        true,
        &["render", "camera", "target", "graph_import"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.ready_for_direct_import",
        false,
        &["render", "camera", "target", "graph_import", "ready"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.requires_conversion_writeback",
        false,
        &["render", "camera", "target", "graph_import", "conversion"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.blocked_format_mismatch",
        false,
        &["render", "camera", "target", "graph_import", "blocked"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.graph_import.direct_import_count",
        0.0,
        &["render", "camera", "target", "graph_import"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.graph_import.conversion_writeback_count",
        0.0,
        &["render", "camera", "target", "graph_import"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.not_requested",
        true,
        &["render", "camera", "target", "writeback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.copied",
        false,
        &["render", "camera", "target", "writeback", "copied"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.ready_for_conversion",
        false,
        &["render", "camera", "target", "writeback", "ready"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.converted",
        false,
        &["render", "camera", "target", "writeback", "converted"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.blocked_format_mismatch",
        false,
        &["render", "camera", "target", "writeback", "blocked"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.copy_count",
        0.0,
        &["render", "camera", "target", "writeback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.converted_count",
        0.0,
        &["render", "camera", "target", "writeback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.debug_marker_emitted",
        false,
        &["render", "camera", "target", "writeback", "debug_marker"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.width",
        0.0,
        &["render", "camera", "target", "writeback", "extent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.height",
        0.0,
        &["render", "camera", "target", "writeback", "extent"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capture.source.none",
        true,
        &["render", "capture", "source"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capture.source.framework_offscreen",
        false,
        &["render", "capture", "source", "framework_offscreen"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capture.width",
        0.0,
        &["render", "capture", "extent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capture.height",
        0.0,
        &["render", "capture", "extent"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.no_previous_frame",
        false,
        &["history", "invalidation", "no_previous_frame"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.frame_inputs_changed",
        false,
        &["history", "invalidation", "frame_inputs_changed"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.render_size_changed",
        true,
        &["history", "invalidation", "render_size_changed"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.last_graph_executed_pass_count",
        14.0,
        &["graph"],
    );
    assert_render_count_series(&snapshot.store, "render.graph.pass_count", 18.0, &["graph"]);
    assert_render_count_series(
        &snapshot.store,
        "render.graph.culled_pass_count",
        4.0,
        &["graph", "culling"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.queue_fallback_pass_count",
        2.0,
        &["graph", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.resource_lifetime_count",
        6.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.sparse_texture_lifetime_count",
        1.0,
        &["graph", "resource", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.planned_resource_access_count",
        22.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.planned_dependency_count",
        9.0,
        &["graph", "dependency"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.transient_texture_slot_count",
        3.0,
        &["graph", "transient", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.sparse_texture_slot_count",
        1.0,
        &["graph", "transient", "texture", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.transient_buffer_slot_count",
        2.0,
        &["graph", "transient", "buffer"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.transient_texture_bytes_reserved",
        4_194_304.0,
        &["graph", "transient", "texture"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.transient_buffer_bytes_reserved",
        65_536.0,
        &["graph", "transient", "buffer"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.transient_dense_bytes_reserved",
        4_259_840.0,
        &["graph", "transient"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.sparse_texture_virtual_bytes",
        16_777_216.0,
        &["graph", "transient", "texture", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_pass_count",
        14.0,
        &["graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_resource_access_count",
        19.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_dependency_count",
        8.0,
        &["graph", "dependency"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.texture_view_count",
        18.0,
        &["graph", "execution", "resource", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.external_texture_view_count",
        14.0,
        &["graph", "execution", "resource", "texture", "external"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.owned_texture_count",
        4.0,
        &["graph", "execution", "resource", "texture", "owned"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.buffer_count",
        3.0,
        &["graph", "execution", "resource", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.bound_resource_count",
        21.0,
        &["graph", "execution", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_created_count",
        5.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "created",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_reused_count",
        7.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "reused",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_created_count",
        2.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "buffer",
            "created",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_reused_count",
        3.0,
        &["graph", "execution", "resource", "pool", "buffer", "reused"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_pool_entry_count",
        4.0,
        &["graph", "execution", "resource", "pool", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_pool_entry_count",
        1.0,
        &["graph", "execution", "resource", "pool", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.evicted_texture_count",
        8.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "evicted",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.evicted_buffer_count",
        9.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "buffer",
            "evicted",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.planned_live_pass_count",
        14.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.executed_pass_count",
        14.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.matched_planned_pass_count",
        14.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.missing_planned_pass_count",
        0.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.unexpected_executed_pass_count",
        0.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.duplicate_executed_pass_count",
        0.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.staged_pass_count",
        14.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.unstaged_pass_count",
        1.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.unique_stage_count",
        7.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.transition_count",
        6.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.order_violation_count",
        0.0,
        &["graph", "execution", "stage", "order"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_dispatch_count",
        2.0,
        &["graph", "compute", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_dispatch_group_count",
        1234.0,
        &["graph", "compute", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_storage_write_resource_count",
        2.0,
        &["graph", "compute", "storage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_planned_workload_count",
        2.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_matched_workload_count",
        1.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_missing_dispatch_count",
        1.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_workload_mismatch_count",
        0.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_unexpected_dispatch_count",
        0.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.debug_marker_count",
        14.0,
        &["graph", "debug_marker"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_anti_alias_pass_count",
        1.0,
        &["graph", "anti_alias"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_virtual_geometry_pass_count",
        2.0,
        &["graph", "virtual_geometry"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_hybrid_gi_pass_count",
        3.0,
        &["graph", "hybrid_gi"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_particle_pass_count",
        1.0,
        &["graph", "particle"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_shadow_pass_count",
        1.0,
        &["graph", "shadow"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_transparent_pass_count",
        4.0,
        &["graph", "transparent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_async_compute_pass_count",
        2.0,
        &["graph", "async_compute"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.requested_requires_history",
        true,
        &["anti_alias", "requested", "history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.effective_post_process",
        true,
        &["anti_alias", "effective"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.fallback.active",
        true,
        &["anti_alias", "fallback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.fallback.missing_history",
        true,
        &["anti_alias", "fallback", "history"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.gpu.alive_count",
        31.0,
        &["particle", "gpu"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.gpu.indirect_instance_count",
        29.0,
        &["particle", "gpu", "indirect"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.node_count",
        5.0,
        &["post_process", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.skipped_node_count",
        1.0,
        &["post_process", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.executed_node_count",
        3.0,
        &["post_process", "graph"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.graph.final_composite_present",
        true,
        &["post_process", "graph", "final_composite"],
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.enabled",
        1.0,
        "bool",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.active_family_count",
        3.0,
        "count",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.approximated_family_count",
        2.0,
        "count",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.missing_resource_count",
        1.0,
        "count",
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.request_count",
        1.0,
        &["post_process", "lut"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.ready_count",
        0.0,
        &["post_process", "lut", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.fallback_count",
        1.0,
        &["post_process", "lut", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.texture_2d_strip_ready_count",
        0.0,
        &["post_process", "lut", "texture_2d_strip", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.texture_3d_request_count",
        1.0,
        &["post_process", "lut", "texture_3d"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.unsupported_shape_count",
        0.0,
        &["post_process", "lut", "unsupported_shape"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.not_requested",
        false,
        &["post_process", "motion_vector", "camera"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.missing_previous_camera",
        false,
        &["post_process", "motion_vector", "camera"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.cut_or_invalid",
        false,
        &["post_process", "motion_vector", "camera"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.ready",
        true,
        &["post_process", "motion_vector", "camera", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.count",
        13.0,
        &["material"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.ready_count",
        10.0,
        &["material"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.fallback_count",
        2.0,
        &["material", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.validation_error_count",
        1.0,
        &["material", "validation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.diagnostic_count",
        4.0,
        &["material", "diagnostic"],
    );
    assert_light_family_series(&snapshot.store, "directional", 3.0, 1.0, 2.0);
    assert_light_family_series(&snapshot.store, "point", 4.0, 0.0, 4.0);
    assert_light_family_series(&snapshot.store, "spot", 5.0, 0.0, 5.0);
    assert_light_family_series(&snapshot.store, "ambient", 2.0, 2.0, 0.0);
    assert_light_family_series(&snapshot.store, "rect", 1.0, 0.0, 1.0);
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.draw_count",
        12.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.opaque_draw_count",
        6.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.alpha_mask_draw_count",
        2.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.transparent_draw_count",
        4.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.early_z_draw_count",
        8.0,
        &["mesh", "queue", "early_z"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.shadow_caster_draw_count",
        8.0,
        &["mesh", "queue", "shadow"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.alpha_mask_shadow_caster_draw_count",
        2.0,
        &["mesh", "queue", "shadow", "alpha_mask"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.prepared_geometry_draw_count",
        5.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_geometry_draw_count",
        7.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_draw_count",
        3.0,
        &["mesh", "queue", "skinned"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_source_candidate_count",
        1.0,
        &["mesh", "queue", "skinned", "gpu_source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
        1.0,
        &["mesh", "queue", "skinned", "gpu_source", "cpu_morphed"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_skinning_draw_count",
        1.0,
        &["mesh", "queue", "skinned", "gpu_skinning"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_draw_count",
        3.0,
        &["mesh", "queue", "indirect"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.lod_draw_count",
        2.0,
        &["mesh", "queue", "lod"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.static_batch_candidate_group_count",
        2.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.static_batch_candidate_draw_count",
        5.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_batch_candidate_group_count",
        3.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_batch_candidate_draw_count",
        6.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_instancing_candidate_group_count",
        4.0,
        &["mesh", "queue", "instancing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_instancing_candidate_draw_count",
        9.0,
        &["mesh", "queue", "instancing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_batch_count",
        2.0,
        &["mesh", "queue", "indirect", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_batched_draw_count",
        5.0,
        &["mesh", "queue", "indirect", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_fallback_draw_count",
        4.0,
        &["mesh", "queue", "indirect", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_args_count",
        5.0,
        &["mesh", "queue", "indirect", "args"],
    );
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
        "render.hybrid_gi.payload.source.authored",
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

    let devtools = crate::core::diagnostics::collect_runtime_devtools_snapshot(&runtime.handle());
    assert!(devtools
        .modules
        .iter()
        .any(|module| module.name == DIAGNOSTICS_TEST_MODULE));
    assert!(devtools
        .services
        .iter()
        .any(|service| service.name == crate::core::manager::RENDER_FRAMEWORK_NAME));
    assert!(devtools
        .plugin_catalog
        .iter()
        .any(|plugin| plugin.package_id == "physics"));
    assert!(devtools
        .diagnostics_summary
        .tagged_subsystems
        .contains(&"render".to_string()));
}
