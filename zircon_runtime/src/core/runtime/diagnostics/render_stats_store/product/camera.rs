use crate::core::framework::render::{
    RenderCameraTargetGraphImportStatus, RenderCameraTargetKind, RenderCameraTargetWritebackStatus,
    RenderCaptureSource, RenderStats,
};

use super::{record_bool, record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_camera_target_resolution(store, frame_index, stats);
    record_camera_target_graph_import(store, frame_index, stats);
    record_camera_target_writeback(store, frame_index, stats);
    record_capture_report(store, frame_index, stats);
    record_count(
        store,
        "render.camera.loop_submission_count",
        frame_index,
        stats.last_camera_loop_submission_count,
        &["render", "camera", "execution"],
    );
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
        "render.camera.target.graph_import.suppressed_by_camera_stack",
        frame_index,
        status == RenderCameraTargetGraphImportStatus::SuppressedByCameraStack,
        &["render", "camera", "target", "graph_import", "camera_stack"],
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
        "render.camera.target.writeback.suppressed_by_camera_stack",
        frame_index,
        status == RenderCameraTargetWritebackStatus::SuppressedByCameraStack,
        &["render", "camera", "target", "writeback", "camera_stack"],
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
