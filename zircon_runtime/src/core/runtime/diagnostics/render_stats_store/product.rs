use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderCameraTargetGraphImportStatus, RenderCameraTargetKind,
    RenderCameraTargetWritebackStatus, RenderCaptureSource, RenderGpuSceneUploadPath, RenderStats,
};

use super::{record_bool, record_bytes, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_camera(store, stats);
    record_visibility(store, stats);
    record_hzb(store, stats);
    record_light_grid(store, stats);
    record_material(store, stats);
    record_light(store, stats);
    record_mesh_queue(store, stats);
    record_gpu_scene(store, stats);
    record_sprite(store, stats);
    record_effect_stack(store, stats);
    record_ui(store, stats);
}

fn record_camera(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_camera_target_resolution(store, frame_index, stats);
    record_camera_target_graph_import(store, frame_index, stats);
    record_camera_target_writeback(store, frame_index, stats);
    record_capture_report(store, frame_index, stats);
    record_count(
        store,
        "render.camera.scheduled_count",
        frame_index,
        stats.last_scene_camera_scheduled_count,
        &["render", "camera", "ordering"],
    );
    record_count(
        store,
        "render.camera.order_ambiguity_count",
        frame_index,
        stats.last_scene_camera_order_ambiguity_count,
        &["render", "camera", "ordering"],
    );
}

fn record_visibility(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.visibility.view_count",
        frame_index,
        stats.last_visibility_view_count,
        &["render", "visibility"],
    );
    record_count(
        store,
        "render.visibility.input_count",
        frame_index,
        stats.last_visibility_input_count,
        &["render", "visibility"],
    );
    record_count(
        store,
        "render.visibility.layer_filtered_count",
        frame_index,
        stats.last_visibility_layer_filtered_count,
        &["render", "visibility", "layer"],
    );
    record_count(
        store,
        "render.visibility.frustum_culled_count",
        frame_index,
        stats.last_visibility_frustum_culled_count,
        &["render", "visibility", "frustum"],
    );
    record_count(
        store,
        "render.visibility.occlusion_culled_count",
        frame_index,
        stats.last_visibility_occlusion_culled_count,
        &["render", "visibility", "occlusion"],
    );
    record_count(
        store,
        "render.visibility.visible_count",
        frame_index,
        stats.last_visibility_visible_count,
        &["render", "visibility", "visible"],
    );
    record_count(
        store,
        "render.visibility.static_index.full_rebuild_count",
        frame_index,
        stats.last_visibility_static_index_full_rebuild_count,
        &["render", "visibility", "static_index", "rebuild"],
    );
    record_count(
        store,
        "render.visibility.static_index.incremental_update_count",
        frame_index,
        stats.last_visibility_static_index_incremental_update_count,
        &["render", "visibility", "static_index", "incremental"],
    );
    record_count(
        store,
        "render.visibility.static_index.inserted_count",
        frame_index,
        stats.last_visibility_static_index_inserted_count,
        &["render", "visibility", "static_index", "change"],
    );
    record_count(
        store,
        "render.visibility.static_index.updated_count",
        frame_index,
        stats.last_visibility_static_index_updated_count,
        &["render", "visibility", "static_index", "change"],
    );
    record_count(
        store,
        "render.visibility.static_index.removed_count",
        frame_index,
        stats.last_visibility_static_index_removed_count,
        &["render", "visibility", "static_index", "change"],
    );
    record_count(
        store,
        "render.visibility.static_index.indexed_entity_count",
        frame_index,
        stats.last_visibility_static_index_indexed_entity_count,
        &["render", "visibility", "static_index", "entity"],
    );
    record_count(
        store,
        "render.visibility.static_index.occupied_cell_count",
        frame_index,
        stats.last_visibility_static_index_occupied_cell_count,
        &["render", "visibility", "static_index", "cell"],
    );
    record_bool(
        store,
        "render.visibility.static_index.main_view_prefilter_used",
        frame_index,
        stats.last_visibility_static_index_main_view_prefilter_used,
        &[
            "render",
            "visibility",
            "static_index",
            "main_view",
            "prefilter",
        ],
    );
    record_count(
        store,
        "render.visibility.static_index.main_view_static_input_count",
        frame_index,
        stats.last_visibility_static_index_main_view_static_input_count,
        &["render", "visibility", "static_index", "main_view", "input"],
    );
    record_count(
        store,
        "render.visibility.static_index.main_view_static_candidate_count",
        frame_index,
        stats.last_visibility_static_index_main_view_static_candidate_count,
        &[
            "render",
            "visibility",
            "static_index",
            "main_view",
            "candidate",
        ],
    );
}

fn record_hzb(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hzb.mip_count",
        frame_index,
        stats.last_hzb_mip_count,
        &["render", "hzb", "mip"],
    );
    record_count(
        store,
        "render.hzb.graph_executed_pass_count",
        frame_index,
        stats.last_hzb_graph_executed_pass_count,
        &["render", "hzb", "graph"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.reported",
        frame_index,
        stats.last_hzb_occlusion_reported,
        &["render", "hzb", "occlusion"],
    );
    record_count(
        store,
        "render.hzb.occlusion.candidate_arg_count",
        frame_index,
        stats.last_hzb_occlusion_candidate_arg_count,
        &["render", "hzb", "occlusion", "candidate"],
    );
    record_count(
        store,
        "render.hzb.occlusion.candidate_instance_count",
        frame_index,
        stats.last_hzb_occlusion_candidate_instance_count,
        &["render", "hzb", "occlusion", "candidate"],
    );
    record_count(
        store,
        "render.hzb.occlusion.dispatch_group_count",
        frame_index,
        stats.last_hzb_occlusion_dispatch_group_count,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    record_count(
        store,
        "render.hzb.occlusion.dispatched_phase_count",
        frame_index,
        stats.last_hzb_occlusion_dispatched_phase_count,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.history_available",
        frame_index,
        stats.last_hzb_occlusion_history_available,
        &["render", "hzb", "occlusion", "history"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.readback_available",
        frame_index,
        stats.last_hzb_occlusion_readback_available,
        &["render", "hzb", "occlusion", "readback"],
    );
    record_count(
        store,
        "render.hzb.occlusion.tested_arg_count",
        frame_index,
        stats.last_hzb_occlusion_tested_arg_count,
        &["render", "hzb", "occlusion", "tested"],
    );
    record_count(
        store,
        "render.hzb.occlusion.tested_instance_count",
        frame_index,
        stats.last_hzb_occlusion_tested_instance_count,
        &["render", "hzb", "occlusion", "tested"],
    );
    record_count(
        store,
        "render.hzb.occlusion.culled_arg_count",
        frame_index,
        stats.last_hzb_occlusion_culled_arg_count,
        &["render", "hzb", "occlusion", "culled"],
    );
    record_count(
        store,
        "render.hzb.occlusion.culled_instance_count",
        frame_index,
        stats.last_hzb_occlusion_culled_instance_count,
        &["render", "hzb", "occlusion", "culled"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.indirect_args_readback_available",
        frame_index,
        stats.last_hzb_occlusion_indirect_args_readback_available,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.readback_arg_count",
        frame_index,
        stats.last_hzb_occlusion_readback_arg_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.compacted_draw_count",
        frame_index,
        stats.last_hzb_occlusion_compacted_draw_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.zero_instance_arg_count",
        frame_index,
        stats.last_hzb_occlusion_zero_instance_arg_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.remaining_instance_count",
        frame_index,
        stats.last_hzb_occlusion_remaining_instance_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
}

fn record_light_grid(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_bool(
        store,
        "render.light_grid.reported",
        frame_index,
        stats.last_light_grid_reported,
        &["render", "light_grid"],
    );
    record_count(
        store,
        "render.light_grid.light_count",
        frame_index,
        stats.last_light_grid_light_count,
        &["render", "light_grid", "light"],
    );
    record_count(
        store,
        "render.light_grid.tile_count",
        frame_index,
        stats.last_light_grid_tile_count,
        &["render", "light_grid", "tile"],
    );
    record_count(
        store,
        "render.light_grid.zbin_count",
        frame_index,
        stats.last_light_grid_zbin_count,
        &["render", "light_grid", "zbin"],
    );
    record_count(
        store,
        "render.light_grid.non_empty_tile_count",
        frame_index,
        stats.last_light_grid_non_empty_tile_count,
        &["render", "light_grid", "tile"],
    );
    record_count(
        store,
        "render.light_grid.non_empty_zbin_count",
        frame_index,
        stats.last_light_grid_non_empty_zbin_count,
        &["render", "light_grid", "zbin"],
    );
    record_count(
        store,
        "render.light_grid.non_empty_cluster_count",
        frame_index,
        stats.last_light_grid_non_empty_cluster_count,
        &["render", "light_grid", "cluster"],
    );
    record_count(
        store,
        "render.light_grid.peak_lights_per_cluster",
        frame_index,
        stats.last_light_grid_peak_lights_per_cluster,
        &["render", "light_grid", "cluster", "peak"],
    );
    store.record(
        "render.light_grid.average_lights_per_cluster",
        frame_index,
        stats.last_light_grid_average_lights_per_cluster_milli as f64 / 1000.0,
        Some("count"),
        ["render", "light_grid", "cluster", "average"],
    );
}

fn record_capture_report(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let report = stats.last_capture_report;
    record_capture_source(store, frame_index, report.source);
    record_count(
        store,
        "render.capture.width",
        frame_index,
        report.output_size.x as usize,
        &["render", "capture", "extent"],
    );
    record_count(
        store,
        "render.capture.height",
        frame_index,
        report.output_size.y as usize,
        &["render", "capture", "extent"],
    );
}

fn record_capture_source(
    store: &mut DiagnosticStore,
    frame_index: u64,
    source: RenderCaptureSource,
) {
    record_bool(
        store,
        "render.capture.source.none",
        frame_index,
        source == RenderCaptureSource::None,
        &["render", "capture", "source"],
    );
    record_bool(
        store,
        "render.capture.source.framework_offscreen",
        frame_index,
        source == RenderCaptureSource::FrameworkOffscreen,
        &["render", "capture", "source", "framework_offscreen"],
    );
    record_bool(
        store,
        "render.capture.source.texture_direct_graph_import",
        frame_index,
        source == RenderCaptureSource::TextureDirectGraphImport,
        &[
            "render",
            "capture",
            "source",
            "texture",
            "direct_graph_import",
        ],
    );
    record_bool(
        store,
        "render.capture.source.texture_writeback_conversion",
        frame_index,
        source == RenderCaptureSource::TextureWritebackConversion,
        &[
            "render",
            "capture",
            "source",
            "texture",
            "writeback",
            "conversion",
        ],
    );
    record_bool(
        store,
        "render.capture.source.texture_writeback_copy",
        frame_index,
        source == RenderCaptureSource::TextureWritebackCopy,
        &[
            "render",
            "capture",
            "source",
            "texture",
            "writeback",
            "copy",
        ],
    );
}

fn record_camera_target_resolution(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = stats.last_camera_target_resolution;
    record_camera_target_kind(store, frame_index, report.target_kind);
    record_count(
        store,
        "render.camera.target.primary_width",
        frame_index,
        report.primary_target_size.x as usize,
        &["render", "camera", "target", "primary"],
    );
    record_count(
        store,
        "render.camera.target.primary_height",
        frame_index,
        report.primary_target_size.y as usize,
        &["render", "camera", "target", "primary"],
    );
    record_count(
        store,
        "render.camera.target.resolved_width",
        frame_index,
        report.resolved_target_size.x as usize,
        &["render", "camera", "target", "resolved"],
    );
    record_count(
        store,
        "render.camera.target.resolved_height",
        frame_index,
        report.resolved_target_size.y as usize,
        &["render", "camera", "target", "resolved"],
    );
    record_count(
        store,
        "render.camera.target.effective_view_width",
        frame_index,
        report.effective_view_size.x as usize,
        &["render", "camera", "target", "effective_view"],
    );
    record_count(
        store,
        "render.camera.target.effective_view_height",
        frame_index,
        report.effective_view_size.y as usize,
        &["render", "camera", "target", "effective_view"],
    );
    record_count(
        store,
        "render.camera.target.effective_render_width",
        frame_index,
        report.effective_render_size.x as usize,
        &["render", "camera", "target", "effective_render"],
    );
    record_count(
        store,
        "render.camera.target.effective_render_height",
        frame_index,
        report.effective_render_size.y as usize,
        &["render", "camera", "target", "effective_render"],
    );
}

fn record_camera_target_kind(
    store: &mut DiagnosticStore,
    frame_index: u64,
    target_kind: RenderCameraTargetKind,
) {
    record_bool(
        store,
        "render.camera.target.primary_surface",
        frame_index,
        target_kind == RenderCameraTargetKind::PrimarySurface,
        &["render", "camera", "target", "primary_surface"],
    );
    record_bool(
        store,
        "render.camera.target.headless",
        frame_index,
        target_kind == RenderCameraTargetKind::Headless,
        &["render", "camera", "target", "headless"],
    );
    record_bool(
        store,
        "render.camera.target.texture",
        frame_index,
        target_kind == RenderCameraTargetKind::Texture,
        &["render", "camera", "target", "texture"],
    );
}

fn record_camera_target_writeback(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = stats.last_camera_target_writeback;
    record_camera_target_writeback_status(store, frame_index, report.status);
    record_count(
        store,
        "render.camera.target.writeback.copy_count",
        frame_index,
        report.copied_count,
        &["render", "camera", "target", "writeback"],
    );
    record_count(
        store,
        "render.camera.target.writeback.converted_count",
        frame_index,
        report.converted_count,
        &["render", "camera", "target", "writeback"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.debug_marker_emitted",
        frame_index,
        report.debug_marker_emitted,
        &["render", "camera", "target", "writeback", "debug_marker"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.conversion_debug_marker_emitted",
        frame_index,
        report.conversion_debug_marker_emitted,
        &[
            "render",
            "camera",
            "target",
            "writeback",
            "debug_marker",
            "conversion",
        ],
    );
    record_count(
        store,
        "render.camera.target.writeback.width",
        frame_index,
        report.target_size.x as usize,
        &["render", "camera", "target", "writeback", "extent"],
    );
    record_count(
        store,
        "render.camera.target.writeback.height",
        frame_index,
        report.target_size.y as usize,
        &["render", "camera", "target", "writeback", "extent"],
    );
}

fn record_camera_target_graph_import(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let report = stats.last_camera_target_graph_import;
    record_camera_target_graph_import_status(store, frame_index, report.status);
    record_count(
        store,
        "render.camera.target.graph_import.direct_import_count",
        frame_index,
        report.direct_import_count,
        &["render", "camera", "target", "graph_import"],
    );
    record_count(
        store,
        "render.camera.target.graph_import.conversion_writeback_count",
        frame_index,
        report.conversion_writeback_count,
        &["render", "camera", "target", "graph_import"],
    );
    record_count(
        store,
        "render.camera.target.graph_import.blocked_count",
        frame_index,
        report.blocked_count,
        &["render", "camera", "target", "graph_import"],
    );
    record_count(
        store,
        "render.camera.target.graph_import.width",
        frame_index,
        report.target_size.x as usize,
        &["render", "camera", "target", "graph_import", "extent"],
    );
    record_count(
        store,
        "render.camera.target.graph_import.height",
        frame_index,
        report.target_size.y as usize,
        &["render", "camera", "target", "graph_import", "extent"],
    );
}

fn record_camera_target_graph_import_status(
    store: &mut DiagnosticStore,
    frame_index: u64,
    status: RenderCameraTargetGraphImportStatus,
) {
    record_bool(
        store,
        "render.camera.target.graph_import.not_requested",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::NotRequested,
        &["render", "camera", "target", "graph_import"],
    );
    record_bool(
        store,
        "render.camera.target.graph_import.pending_target_descriptor",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::PendingTargetDescriptor,
        &["render", "camera", "target", "graph_import"],
    );
    record_bool(
        store,
        "render.camera.target.graph_import.ready_for_direct_import",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::ReadyForDirectImport,
        &["render", "camera", "target", "graph_import", "ready"],
    );
    record_bool(
        store,
        "render.camera.target.graph_import.direct_imported",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::DirectImported,
        &["render", "camera", "target", "graph_import", "direct"],
    );
    record_bool(
        store,
        "render.camera.target.graph_import.requires_conversion_writeback",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::RequiresConversionWriteback,
        &["render", "camera", "target", "graph_import", "conversion"],
    );
    record_bool(
        store,
        "render.camera.target.graph_import.blocked_format_mismatch",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::BlockedFormatMismatch,
        &["render", "camera", "target", "graph_import", "blocked"],
    );
}

fn record_camera_target_writeback_status(
    store: &mut DiagnosticStore,
    frame_index: u64,
    status: RenderCameraTargetWritebackStatus,
) {
    record_bool(
        store,
        "render.camera.target.writeback.not_requested",
        frame_index,
        status == RenderCameraTargetWritebackStatus::NotRequested,
        &["render", "camera", "target", "writeback"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.pending_target_descriptor",
        frame_index,
        status == RenderCameraTargetWritebackStatus::PendingTargetDescriptor,
        &["render", "camera", "target", "writeback"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.ready_for_copy",
        frame_index,
        status == RenderCameraTargetWritebackStatus::ReadyForCopy,
        &["render", "camera", "target", "writeback", "ready"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.ready_for_conversion",
        frame_index,
        status == RenderCameraTargetWritebackStatus::ReadyForConversion,
        &["render", "camera", "target", "writeback", "ready"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.skipped_direct_import",
        frame_index,
        status == RenderCameraTargetWritebackStatus::SkippedDirectImport,
        &["render", "camera", "target", "writeback", "direct_import"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.copied",
        frame_index,
        status == RenderCameraTargetWritebackStatus::Copied,
        &["render", "camera", "target", "writeback", "copied"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.converted",
        frame_index,
        status == RenderCameraTargetWritebackStatus::Converted,
        &["render", "camera", "target", "writeback", "converted"],
    );
    record_bool(
        store,
        "render.camera.target.writeback.blocked_format_mismatch",
        frame_index,
        status == RenderCameraTargetWritebackStatus::BlockedFormatMismatch,
        &["render", "camera", "target", "writeback", "blocked"],
    );
}

fn record_effect_stack(store: &mut DiagnosticStore, stats: &RenderStats) {
    let report = &stats.last_post_process_effect_stack_report;
    let frame_index = stats.submitted_frames;
    store.record(
        "render.post_process.effect_stack.enabled",
        frame_index,
        u8::from(report.enabled) as f64,
        Some("bool"),
        ["render", "post_process", "effect_stack"],
    );
    store.record(
        "render.post_process.effect_stack.active_family_count",
        frame_index,
        report.active_family_count as f64,
        Some("count"),
        ["render", "post_process", "effect_stack"],
    );
    store.record(
        "render.post_process.effect_stack.approximated_family_count",
        frame_index,
        report.approximated_family_count as f64,
        Some("count"),
        ["render", "post_process", "effect_stack"],
    );
    store.record(
        "render.post_process.effect_stack.missing_resource_count",
        frame_index,
        report.missing_resource_count as f64,
        Some("count"),
        ["render", "post_process", "effect_stack"],
    );
    record_count(
        store,
        "render.post_process.lut.request_count",
        frame_index,
        stats.last_post_process_lut_request_count,
        &["render", "post_process", "lut"],
    );
    record_count(
        store,
        "render.post_process.lut.ready_count",
        frame_index,
        stats.last_post_process_lut_ready_count,
        &["render", "post_process", "lut", "ready"],
    );
    record_count(
        store,
        "render.post_process.lut.fallback_count",
        frame_index,
        stats.last_post_process_lut_fallback_count,
        &["render", "post_process", "lut", "fallback"],
    );
    record_count(
        store,
        "render.post_process.lut.texture_2d_strip_ready_count",
        frame_index,
        stats.last_post_process_lut_2d_strip_ready_count,
        &["render", "post_process", "lut", "texture_2d_strip", "ready"],
    );
    record_count(
        store,
        "render.post_process.lut.texture_3d_request_count",
        frame_index,
        stats.last_post_process_lut_3d_request_count,
        &["render", "post_process", "lut", "texture_3d"],
    );
    record_count(
        store,
        "render.post_process.lut.unsupported_shape_count",
        frame_index,
        stats.last_post_process_lut_unsupported_shape_count,
        &["render", "post_process", "lut", "unsupported_shape"],
    );
    record_motion_vector_camera_status(store, frame_index, stats.last_motion_vector_camera_status);
}

fn record_motion_vector_camera_status(
    store: &mut DiagnosticStore,
    frame_index: u64,
    status: MotionVectorCameraStatus,
) {
    record_bool(
        store,
        "render.post_process.motion_vector.camera.not_requested",
        frame_index,
        status == MotionVectorCameraStatus::NotRequested,
        &["render", "post_process", "motion_vector", "camera"],
    );
    record_bool(
        store,
        "render.post_process.motion_vector.camera.missing_previous_camera",
        frame_index,
        status == MotionVectorCameraStatus::MissingPreviousCamera,
        &["render", "post_process", "motion_vector", "camera"],
    );
    record_bool(
        store,
        "render.post_process.motion_vector.camera.cut_or_invalid",
        frame_index,
        status == MotionVectorCameraStatus::CameraCutOrInvalid,
        &["render", "post_process", "motion_vector", "camera"],
    );
    record_bool(
        store,
        "render.post_process.motion_vector.camera.ready",
        frame_index,
        status == MotionVectorCameraStatus::Ready,
        &["render", "post_process", "motion_vector", "camera", "ready"],
    );
}

fn record_material(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.material.count",
        frame_index,
        stats.last_material_count,
        &["render", "material"],
    );
    record_count(
        store,
        "render.material.ready_count",
        frame_index,
        stats.last_material_ready_count,
        &["render", "material"],
    );
    record_count(
        store,
        "render.material.fallback_count",
        frame_index,
        stats.last_material_fallback_count,
        &["render", "material", "fallback"],
    );
    record_count(
        store,
        "render.material.validation_error_count",
        frame_index,
        stats.last_material_validation_error_count,
        &["render", "material", "validation"],
    );
    record_count(
        store,
        "render.material.diagnostic_count",
        frame_index,
        stats.last_material_diagnostic_count,
        &["render", "material", "diagnostic"],
    );
}

fn record_light(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_light_family(
        store,
        frame_index,
        "directional",
        stats.last_directional_light_count,
        stats.last_directional_light_ready_count,
        stats.last_directional_light_degraded_count,
        (
            "render.light.directional.count",
            "render.light.directional.ready_count",
            "render.light.directional.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "point",
        stats.last_point_light_count,
        stats.last_point_light_ready_count,
        stats.last_point_light_degraded_count,
        (
            "render.light.point.count",
            "render.light.point.ready_count",
            "render.light.point.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "spot",
        stats.last_spot_light_count,
        stats.last_spot_light_ready_count,
        stats.last_spot_light_degraded_count,
        (
            "render.light.spot.count",
            "render.light.spot.ready_count",
            "render.light.spot.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "ambient",
        stats.last_ambient_light_count,
        stats.last_ambient_light_ready_count,
        stats.last_ambient_light_degraded_count,
        (
            "render.light.ambient.count",
            "render.light.ambient.ready_count",
            "render.light.ambient.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "rect",
        stats.last_rect_light_count,
        stats.last_rect_light_ready_count,
        stats.last_rect_light_degraded_count,
        (
            "render.light.rect.count",
            "render.light.rect.ready_count",
            "render.light.rect.degraded_count",
        ),
    );
}

fn record_mesh_queue(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.mesh.queue.draw_count",
        frame_index,
        stats.last_mesh_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.opaque_draw_count",
        frame_index,
        stats.last_mesh_opaque_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.alpha_mask_draw_count",
        frame_index,
        stats.last_mesh_alpha_mask_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.transparent_draw_count",
        frame_index,
        stats.last_mesh_transparent_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.early_z_draw_count",
        frame_index,
        stats.last_mesh_early_z_draw_count,
        &["render", "mesh", "queue", "early_z"],
    );
    record_count(
        store,
        "render.mesh.queue.shadow_caster_draw_count",
        frame_index,
        stats.last_mesh_shadow_caster_draw_count,
        &["render", "mesh", "queue", "shadow"],
    );
    record_count(
        store,
        "render.mesh.queue.alpha_mask_shadow_caster_draw_count",
        frame_index,
        stats.last_mesh_alpha_mask_shadow_caster_draw_count,
        &["render", "mesh", "queue", "shadow", "alpha_mask"],
    );
    record_count(
        store,
        "render.mesh.queue.prepared_geometry_draw_count",
        frame_index,
        stats.last_mesh_prepared_geometry_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_geometry_draw_count",
        frame_index,
        stats.last_mesh_dynamic_geometry_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_draw_count",
        frame_index,
        stats.last_mesh_skinned_draw_count,
        &["render", "mesh", "queue", "skinned"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_palette_upload_count",
        frame_index,
        stats.last_mesh_skinned_palette_upload_count,
        &["render", "mesh", "queue", "skinned", "palette"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_previous_palette_upload_count",
        frame_index,
        stats.last_mesh_skinned_previous_palette_upload_count,
        &["render", "mesh", "queue", "skinned", "palette", "previous"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_source_candidate_count",
        frame_index,
        stats.last_mesh_skinned_gpu_source_candidate_count,
        &["render", "mesh", "queue", "skinned", "gpu_source"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
        frame_index,
        stats.last_mesh_skinned_gpu_cpu_morphed_source_candidate_count,
        &[
            "render",
            "mesh",
            "queue",
            "skinned",
            "gpu_source",
            "cpu_morphed",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
        frame_index,
        stats.last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count,
        &[
            "render",
            "mesh",
            "queue",
            "skinned",
            "gpu_source",
            "cpu_morphed",
            "previous_shape_missing",
            "velocity",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_skinning_draw_count",
        frame_index,
        stats.last_mesh_skinned_gpu_skinning_draw_count,
        &["render", "mesh", "queue", "skinned", "gpu_skinning"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_velocity_draw_count",
        frame_index,
        stats.last_mesh_skinned_gpu_velocity_draw_count,
        &[
            "render",
            "mesh",
            "queue",
            "skinned",
            "gpu_skinning",
            "velocity",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_draw_count",
        frame_index,
        stats.last_mesh_indirect_draw_count,
        &["render", "mesh", "queue", "indirect"],
    );
    record_count(
        store,
        "render.mesh.queue.lod_draw_count",
        frame_index,
        stats.last_mesh_lod_draw_count,
        &["render", "mesh", "queue", "lod"],
    );
    record_count(
        store,
        "render.mesh.queue.previous_velocity_transform_draw_count",
        frame_index,
        stats.last_mesh_previous_velocity_transform_draw_count,
        &["render", "mesh", "queue", "velocity", "previous"],
    );
    record_count(
        store,
        "render.mesh.queue.missing_velocity_transform_draw_count",
        frame_index,
        stats.last_mesh_missing_velocity_transform_draw_count,
        &["render", "mesh", "queue", "velocity", "missing"],
    );
    record_count(
        store,
        "render.mesh.queue.taa_reactive_mask_command_count",
        frame_index,
        stats.last_mesh_taa_reactive_mask_command_count,
        &["render", "mesh", "queue", "taa", "reactive_mask"],
    );
    record_count(
        store,
        "render.mesh.queue.static_batch_candidate_group_count",
        frame_index,
        stats.last_mesh_static_batch_candidate_group_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.static_batch_candidate_draw_count",
        frame_index,
        stats.last_mesh_static_batch_candidate_draw_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_batch_candidate_group_count",
        frame_index,
        stats.last_mesh_dynamic_batch_candidate_group_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_batch_candidate_draw_count",
        frame_index,
        stats.last_mesh_dynamic_batch_candidate_draw_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.gpu_instancing_candidate_group_count",
        frame_index,
        stats.last_mesh_gpu_instancing_candidate_group_count,
        &["render", "mesh", "queue", "instancing"],
    );
    record_count(
        store,
        "render.mesh.queue.gpu_instancing_candidate_draw_count",
        frame_index,
        stats.last_mesh_gpu_instancing_candidate_draw_count,
        &["render", "mesh", "queue", "instancing"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_batch_count",
        frame_index,
        stats.last_indirect_batch_count,
        &["render", "mesh", "queue", "indirect", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_batched_draw_count",
        frame_index,
        stats.last_indirect_batched_draw_count,
        &["render", "mesh", "queue", "indirect", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_fallback_draw_count",
        frame_index,
        stats.last_indirect_fallback_draw_count,
        &["render", "mesh", "queue", "indirect", "fallback"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_args_count",
        frame_index,
        stats.last_indirect_args_count,
        &["render", "mesh", "queue", "indirect", "args"],
    );
}

fn record_gpu_scene(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.gpu_scene.primitive_count",
        frame_index,
        stats.last_gpu_scene_primitive_count as usize,
        &["render", "gpu_scene"],
    );
    record_count(
        store,
        "render.gpu_scene.instance_count",
        frame_index,
        stats.last_gpu_scene_instance_count as usize,
        &["render", "gpu_scene"],
    );
    record_count(
        store,
        "render.gpu_scene.dirty_entry_count",
        frame_index,
        stats.last_gpu_scene_dirty_entry_count,
        &["render", "gpu_scene", "upload"],
    );
    record_bytes(
        store,
        "render.gpu_scene.uploaded_bytes",
        frame_index,
        stats.last_gpu_scene_uploaded_bytes,
        &["render", "gpu_scene", "upload"],
    );
    record_bool(
        store,
        "render.gpu_scene.upload_path.direct_queue_write",
        frame_index,
        stats.last_gpu_scene_upload_path == RenderGpuSceneUploadPath::DirectQueueWrite,
        &["render", "gpu_scene", "upload", "direct_queue_write"],
    );
    record_count(
        store,
        "render.gpu_scene.free_span_count",
        frame_index,
        stats.last_gpu_scene_free_span_count,
        &["render", "gpu_scene", "allocator"],
    );
    record_count(
        store,
        "render.gpu_scene.primitive_upload_range_count",
        frame_index,
        stats.last_gpu_scene_primitive_upload_range_count,
        &["render", "gpu_scene", "upload", "primitive"],
    );
    record_count(
        store,
        "render.gpu_scene.instance_upload_range_count",
        frame_index,
        stats.last_gpu_scene_instance_upload_range_count,
        &["render", "gpu_scene", "upload", "instance"],
    );
}

fn record_sprite(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.sprite.count",
        frame_index,
        stats.last_sprite_count,
        &["render", "sprite"],
    );
    record_count(
        store,
        "render.sprite.ready_count",
        frame_index,
        stats.last_sprite_ready_count,
        &["render", "sprite"],
    );
    record_count(
        store,
        "render.sprite.texture_fallback_count",
        frame_index,
        stats.last_sprite_texture_fallback_count,
        &["render", "sprite", "fallback"],
    );
    record_count(
        store,
        "render.sprite.graph_executed_pass_count",
        frame_index,
        stats.last_sprite_graph_executed_pass_count,
        &["render", "sprite", "graph"],
    );
    record_count(
        store,
        "render.sprite.queue.draw_batch_count",
        frame_index,
        stats.last_sprite_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.batched_sprite_count",
        frame_index,
        stats.last_sprite_batched_sprite_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.image_slice_count",
        frame_index,
        stats.last_sprite_image_slice_count,
        &["render", "sprite", "queue", "image_slice"],
    );
    record_count(
        store,
        "render.sprite.queue.expanded_image_slice_count",
        frame_index,
        stats.last_sprite_expanded_image_slice_count,
        &["render", "sprite", "queue", "image_slice", "expanded"],
    );
    record_count(
        store,
        "render.sprite.queue.vertex_count",
        frame_index,
        stats.last_sprite_vertex_count,
        &["render", "sprite", "queue"],
    );
    record_count(
        store,
        "render.sprite.queue.opaque_draw_batch_count",
        frame_index,
        stats.last_sprite_opaque_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.alpha_mask_draw_batch_count",
        frame_index,
        stats.last_sprite_alpha_mask_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.transparent_draw_batch_count",
        frame_index,
        stats.last_sprite_transparent_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
}

fn record_ui(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.ui.command_count",
        frame_index,
        stats.last_ui_command_count,
        &["render", "ui"],
    );
    record_count(
        store,
        "render.ui.quad_count",
        frame_index,
        stats.last_ui_quad_count,
        &["render", "ui"],
    );
    record_count(
        store,
        "render.ui.text_payload_count",
        frame_index,
        stats.last_ui_text_payload_count,
        &["render", "ui", "text"],
    );
    record_count(
        store,
        "render.ui.image_payload_count",
        frame_index,
        stats.last_ui_image_payload_count,
        &["render", "ui", "image"],
    );
    record_count(
        store,
        "render.ui.clipped_command_count",
        frame_index,
        stats.last_ui_clipped_command_count,
        &["render", "ui", "clip"],
    );
    record_count(
        store,
        "render.ui.graph_executed_pass_count",
        frame_index,
        stats.last_ui_graph_executed_pass_count,
        &["render", "ui", "graph"],
    );
}

fn record_light_family(
    store: &mut DiagnosticStore,
    frame_index: u64,
    family_tag: &'static str,
    count: usize,
    ready_count: usize,
    degraded_count: usize,
    paths: (&'static str, &'static str, &'static str),
) {
    record_count(
        store,
        paths.0,
        frame_index,
        count,
        &["render", "light", family_tag],
    );
    record_count(
        store,
        paths.1,
        frame_index,
        ready_count,
        &["render", "light", family_tag, "ready"],
    );
    record_count(
        store,
        paths.2,
        frame_index,
        degraded_count,
        &["render", "light", family_tag, "degraded"],
    );
}

#[cfg(test)]
mod tests {
    use crate::core::diagnostics::DiagnosticStore;
    use crate::core::framework::render::{
        RenderCameraTargetGraphImportReport, RenderCameraTargetKind,
        RenderCameraTargetWritebackReport, RenderCaptureReport, RenderCaptureSource,
        RenderGpuSceneUploadPath, RenderStats,
    };
    use crate::core::math::UVec2;

    use super::record;

    #[test]
    fn render_product_diagnostics_record_texture_conversion_writeback_marker() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_camera_target_writeback: RenderCameraTargetWritebackReport::converted(UVec2::new(
                72, 40,
            )),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.camera.target.writeback.converted",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.converted_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.conversion_debug_marker_emitted",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.debug_marker_emitted",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.width",
            72.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.height",
            40.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_texture_direct_graph_import_readiness() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_camera_target_graph_import:
                RenderCameraTargetGraphImportReport::ready_for_direct_import(UVec2::new(96, 54)),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.camera.target.graph_import.ready_for_direct_import",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_imported",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.requires_conversion_writeback",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_import_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.conversion_writeback_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.width",
            96.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.height",
            54.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_texture_direct_graph_import_execution() {
        let mut store = DiagnosticStore::default();
        let stats =
            RenderStats {
                submitted_frames: 12,
                last_camera_target_graph_import:
                    RenderCameraTargetGraphImportReport::direct_imported(UVec2::new(96, 54)),
                last_camera_target_writeback:
                    RenderCameraTargetWritebackReport::skipped_direct_import(UVec2::new(96, 54)),
                ..RenderStats::default()
            };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.camera.target.graph_import.ready_for_direct_import",
            0.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_imported",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.graph_import.direct_import_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.skipped_direct_import",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.camera.target.writeback.copy_count",
            0.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_capture_source_report() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_capture_report: RenderCaptureReport::new(
                RenderCameraTargetKind::Texture,
                RenderCaptureSource::TextureWritebackConversion,
                UVec2::new(72, 40),
                crate::core::framework::render::RenderCameraTargetGraphImportStatus::RequiresConversionWriteback,
                crate::core::framework::render::RenderCameraTargetWritebackStatus::Converted,
            ),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.capture.source.texture_writeback_conversion",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.capture.source.texture_direct_graph_import",
            0.0,
            "bool",
        );
        assert_series(&store, "render.capture.width", 72.0, "count");
        assert_series(&store, "render.capture.height", 40.0, "count");
    }

    #[test]
    fn render_product_diagnostics_record_visibility_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_visibility_view_count: 2,
            last_visibility_input_count: 8,
            last_visibility_layer_filtered_count: 1,
            last_visibility_frustum_culled_count: 3,
            last_visibility_occlusion_culled_count: 1,
            last_visibility_visible_count: 3,
            last_visibility_static_index_full_rebuild_count: 0,
            last_visibility_static_index_incremental_update_count: 1,
            last_visibility_static_index_inserted_count: 2,
            last_visibility_static_index_updated_count: 3,
            last_visibility_static_index_removed_count: 4,
            last_visibility_static_index_indexed_entity_count: 10,
            last_visibility_static_index_occupied_cell_count: 7,
            last_visibility_static_index_main_view_prefilter_used: true,
            last_visibility_static_index_main_view_static_input_count: 12,
            last_visibility_static_index_main_view_static_candidate_count: 5,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.visibility.view_count", 2.0, "count");
        assert_series(&store, "render.visibility.input_count", 8.0, "count");
        assert_series(
            &store,
            "render.visibility.layer_filtered_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.frustum_culled_count",
            3.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.occlusion_culled_count",
            1.0,
            "count",
        );
        assert_series(&store, "render.visibility.visible_count", 3.0, "count");
        assert_series(
            &store,
            "render.visibility.static_index.full_rebuild_count",
            0.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.incremental_update_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.inserted_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.updated_count",
            3.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.removed_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.indexed_entity_count",
            10.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.occupied_cell_count",
            7.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.main_view_prefilter_used",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.visibility.static_index.main_view_static_input_count",
            12.0,
            "count",
        );
        assert_series(
            &store,
            "render.visibility.static_index.main_view_static_candidate_count",
            5.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_hzb_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_hzb_mip_count: 10,
            last_hzb_graph_executed_pass_count: 1,
            last_hzb_occlusion_reported: true,
            last_hzb_occlusion_candidate_arg_count: 6,
            last_hzb_occlusion_candidate_instance_count: 42,
            last_hzb_occlusion_dispatch_group_count: 2,
            last_hzb_occlusion_dispatched_phase_count: 1,
            last_hzb_occlusion_history_available: true,
            last_hzb_occlusion_readback_available: true,
            last_hzb_occlusion_tested_arg_count: 6,
            last_hzb_occlusion_tested_instance_count: 42,
            last_hzb_occlusion_culled_arg_count: 2,
            last_hzb_occlusion_culled_instance_count: 18,
            last_hzb_occlusion_indirect_args_readback_available: true,
            last_hzb_occlusion_readback_arg_count: 6,
            last_hzb_occlusion_compacted_draw_count: 4,
            last_hzb_occlusion_zero_instance_arg_count: 2,
            last_hzb_occlusion_remaining_instance_count: 24,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.hzb.mip_count", 10.0, "count");
        assert_series(&store, "render.hzb.graph_executed_pass_count", 1.0, "count");
        assert_series(&store, "render.hzb.occlusion.reported", 1.0, "bool");
        assert_series(
            &store,
            "render.hzb.occlusion.candidate_arg_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.candidate_instance_count",
            42.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.dispatch_group_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.dispatched_phase_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.history_available",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.readback_available",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.tested_arg_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.tested_instance_count",
            42.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.culled_arg_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.culled_instance_count",
            18.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.indirect_args_readback_available",
            1.0,
            "bool",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.readback_arg_count",
            6.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.compacted_draw_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.zero_instance_arg_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.hzb.occlusion.remaining_instance_count",
            24.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_light_grid_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_light_grid_reported: true,
            last_light_grid_light_count: 9,
            last_light_grid_tile_count: 64,
            last_light_grid_zbin_count: 32,
            last_light_grid_non_empty_tile_count: 11,
            last_light_grid_non_empty_zbin_count: 7,
            last_light_grid_non_empty_cluster_count: 23,
            last_light_grid_peak_lights_per_cluster: 5,
            last_light_grid_average_lights_per_cluster_milli: 375,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.light_grid.reported", 1.0, "bool");
        assert_series(&store, "render.light_grid.light_count", 9.0, "count");
        assert_series(&store, "render.light_grid.tile_count", 64.0, "count");
        assert_series(&store, "render.light_grid.zbin_count", 32.0, "count");
        assert_series(
            &store,
            "render.light_grid.non_empty_tile_count",
            11.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.non_empty_zbin_count",
            7.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.non_empty_cluster_count",
            23.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.peak_lights_per_cluster",
            5.0,
            "count",
        );
        assert_series(
            &store,
            "render.light_grid.average_lights_per_cluster",
            0.375,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_skinned_mesh_queue_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_skinned_draw_count: 3,
            last_mesh_skinned_palette_upload_count: 2,
            last_mesh_skinned_previous_palette_upload_count: 1,
            last_mesh_skinned_gpu_source_candidate_count: 1,
            last_mesh_skinned_gpu_cpu_morphed_source_candidate_count: 1,
            last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: 1,
            last_mesh_skinned_gpu_skinning_draw_count: 1,
            last_mesh_skinned_gpu_velocity_draw_count: 1,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.mesh.queue.skinned_draw_count", 3.0, "count");
        assert_series(
            &store,
            "render.mesh.queue.skinned_palette_upload_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_previous_palette_upload_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_source_candidate_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_skinning_draw_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.skinned_gpu_velocity_draw_count",
            1.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_gpu_scene_upload_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_gpu_scene_primitive_count: 5,
            last_gpu_scene_instance_count: 7,
            last_gpu_scene_dirty_entry_count: 3,
            last_gpu_scene_uploaded_bytes: 128,
            last_gpu_scene_upload_path: RenderGpuSceneUploadPath::DirectQueueWrite,
            last_gpu_scene_free_span_count: 2,
            last_gpu_scene_primitive_upload_range_count: 1,
            last_gpu_scene_instance_upload_range_count: 4,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.gpu_scene.primitive_count", 5.0, "count");
        assert_series(&store, "render.gpu_scene.instance_count", 7.0, "count");
        assert_series(&store, "render.gpu_scene.dirty_entry_count", 3.0, "count");
        assert_series(&store, "render.gpu_scene.uploaded_bytes", 128.0, "bytes");
        assert_series(
            &store,
            "render.gpu_scene.upload_path.direct_queue_write",
            1.0,
            "bool",
        );
        assert_series(&store, "render.gpu_scene.free_span_count", 2.0, "count");
        assert_series(
            &store,
            "render.gpu_scene.primitive_upload_range_count",
            1.0,
            "count",
        );
        assert_series(
            &store,
            "render.gpu_scene.instance_upload_range_count",
            4.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_mesh_indirect_batch_stats() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_indirect_batch_count: 2,
            last_indirect_batched_draw_count: 5,
            last_indirect_fallback_draw_count: 4,
            last_indirect_args_count: 5,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.mesh.queue.indirect_batch_count",
            2.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.indirect_batched_draw_count",
            5.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.indirect_fallback_draw_count",
            4.0,
            "count",
        );
        assert_series(
            &store,
            "render.mesh.queue.indirect_args_count",
            5.0,
            "count",
        );
    }

    #[test]
    fn render_product_diagnostics_record_mesh_lod_queue_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_lod_draw_count: 4,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(&store, "render.mesh.queue.lod_draw_count", 4.0, "count");
    }

    #[test]
    fn render_product_diagnostics_record_taa_reactive_mask_queue_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_mesh_taa_reactive_mask_command_count: 3,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.mesh.queue.taa_reactive_mask_command_count",
            3.0,
            "count",
        );
    }

    fn assert_series(store: &DiagnosticStore, path: &str, value: f64, unit: &str) {
        let snapshot = store.snapshot();
        let series = snapshot
            .series
            .iter()
            .find(|series| series.path.as_str() == path)
            .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
        assert_eq!(series.current, Some(value));
        assert_eq!(series.unit.as_deref(), Some(unit));
        assert_eq!(series.history.len(), 1);
        assert_eq!(series.history[0].frame_index, 12);
        assert_eq!(series.history[0].value, value);
    }
}
