use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderCameraTargetGraphImportStatus, RenderCameraTargetKind,
    RenderCameraTargetWritebackStatus, RenderCaptureSource, RenderStats,
};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_camera(store, stats);
    record_material(store, stats);
    record_light(store, stats);
    record_mesh_queue(store, stats);
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
    record_motion_vector_object_history(store, frame_index, stats);
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

fn record_motion_vector_object_history(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    record_count(
        store,
        "render.post_process.motion_vector.object.previous_history_count",
        frame_index,
        stats.last_motion_vector_previous_object_history_count,
        &[
            "render",
            "post_process",
            "motion_vector",
            "object",
            "history",
        ],
    );
    record_count(
        store,
        "render.post_process.motion_vector.object.current_history_count",
        frame_index,
        stats.last_motion_vector_current_object_history_count,
        &[
            "render",
            "post_process",
            "motion_vector",
            "object",
            "history",
        ],
    );
    record_count(
        store,
        "render.post_process.motion_vector.object.matched_history_count",
        frame_index,
        stats.last_motion_vector_matched_object_history_count,
        &[
            "render",
            "post_process",
            "motion_vector",
            "object",
            "history",
            "matched",
        ],
    );
    record_count(
        store,
        "render.post_process.motion_vector.object.missing_history_count",
        frame_index,
        stats.last_motion_vector_missing_object_history_count,
        &[
            "render",
            "post_process",
            "motion_vector",
            "object",
            "history",
            "missing",
        ],
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
        "render.mesh.queue.indirect_draw_count",
        frame_index,
        stats.last_mesh_indirect_draw_count,
        &["render", "mesh", "queue", "indirect"],
    );
    record_count(
        store,
        "render.mesh.queue.previous_motion_vector_transform_draw_count",
        frame_index,
        stats.last_mesh_previous_motion_vector_transform_draw_count,
        &["render", "mesh", "queue", "motion_vector", "previous"],
    );
    record_count(
        store,
        "render.mesh.queue.missing_motion_vector_transform_draw_count",
        frame_index,
        stats.last_mesh_missing_motion_vector_transform_draw_count,
        &["render", "mesh", "queue", "motion_vector", "missing"],
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
        RenderCameraTargetWritebackReport, RenderCaptureReport, RenderCaptureSource, RenderStats,
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
