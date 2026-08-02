use crate::core::framework::render::{FrameHistoryInvalidationReason, RenderStats};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let history = stats.last_frame_history_status;
    record_bool(
        store,
        "render.history.current_handle_present",
        frame_index,
        history.current.is_some(),
        &["render", "history"],
    );
    record_bool(
        store,
        "render.history.previous_handle_present",
        frame_index,
        history.previous.is_some(),
        &["render", "history"],
    );
    record_bool(
        store,
        "render.history.previous_available",
        frame_index,
        history.previous_available,
        &["render", "history"],
    );
    record_bool(
        store,
        "render.history.invalidated",
        frame_index,
        history.invalidation_reason.is_some(),
        &["render", "history", "invalidation"],
    );
    record_count(
        store,
        "render.history.target_width",
        frame_index,
        history.target_size.x as usize,
        &["render", "history", "target_size"],
    );
    record_count(
        store,
        "render.history.target_height",
        frame_index,
        history.target_size.y as usize,
        &["render", "history", "target_size"],
    );
    record_count(
        store,
        "render.history.render_width",
        frame_index,
        history.render_size.x as usize,
        &["render", "history", "render_size"],
    );
    record_count(
        store,
        "render.history.render_height",
        frame_index,
        history.render_size.y as usize,
        &["render", "history", "render_size"],
    );
    record_history_copy_report(store, stats);
    record_invalidation_reason(
        store,
        frame_index,
        history.invalidation_reason,
        FrameHistoryInvalidationReason::NoPreviousFrame,
        "render.history.invalidated.no_previous_frame",
        "no_previous_frame",
    );
    record_invalidation_reason(
        store,
        frame_index,
        history.invalidation_reason,
        FrameHistoryInvalidationReason::ViewportResized,
        "render.history.invalidated.viewport_resized",
        "viewport_resized",
    );
    record_invalidation_reason(
        store,
        frame_index,
        history.invalidation_reason,
        FrameHistoryInvalidationReason::RenderSizeChanged,
        "render.history.invalidated.render_size_changed",
        "render_size_changed",
    );
    record_invalidation_reason(
        store,
        frame_index,
        history.invalidation_reason,
        FrameHistoryInvalidationReason::PipelineChanged,
        "render.history.invalidated.pipeline_changed",
        "pipeline_changed",
    );
    record_invalidation_reason(
        store,
        frame_index,
        history.invalidation_reason,
        FrameHistoryInvalidationReason::HistoryBindingChanged,
        "render.history.invalidated.history_binding_changed",
        "history_binding_changed",
    );
    record_invalidation_reason(
        store,
        frame_index,
        history.invalidation_reason,
        FrameHistoryInvalidationReason::FrameInputsChanged,
        "render.history.invalidated.frame_inputs_changed",
        "frame_inputs_changed",
    );
}

fn record_history_copy_report(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = stats.last_frame_history_copy_report;
    record_bool(
        store,
        "render.history.copy.history_target_present",
        frame_index,
        report.history_target_present,
        &["render", "history", "copy"],
    );
    record_bool(
        store,
        "render.history.copy.debug_marker_emitted",
        frame_index,
        report.debug_marker_emitted,
        &["render", "history", "copy", "debug_marker"],
    );
    record_count(
        store,
        "render.history.copy.requested_count",
        frame_index,
        report.requested_copy_count,
        &["render", "history", "copy"],
    );
    record_count(
        store,
        "render.history.copy.copied_count",
        frame_index,
        report.copied_count,
        &["render", "history", "copy"],
    );
    record_count(
        store,
        "render.history.copy.target_width",
        frame_index,
        report.target_size.x as usize,
        &["render", "history", "copy", "target_size"],
    );
    record_count(
        store,
        "render.history.copy.target_height",
        frame_index,
        report.target_size.y as usize,
        &["render", "history", "copy", "target_size"],
    );
    record_bool(
        store,
        "render.history.copy.scene_color_copied",
        frame_index,
        report.scene_color_copied,
        &["render", "history", "copy", "scene_color"],
    );
    record_bool(
        store,
        "render.history.copy.global_illumination_copied",
        frame_index,
        report.global_illumination_copied,
        &["render", "history", "copy", "global_illumination"],
    );
    record_bool(
        store,
        "render.history.copy.ambient_occlusion_copied",
        frame_index,
        report.ambient_occlusion_copied,
        &["render", "history", "copy", "ambient_occlusion"],
    );
    record_bool(
        store,
        "render.history.copy.screen_space_reflection_copied",
        frame_index,
        report.screen_space_reflection_copied,
        &["render", "history", "copy", "screen_space_reflection"],
    );
    record_bool(
        store,
        "render.history.copy.hzb_furthest_copied",
        frame_index,
        report.hzb_furthest_copied,
        &["render", "history", "copy", "hzb"],
    );
    record_bool(
        store,
        "render.history.copy.exposure_copied",
        frame_index,
        report.exposure_copied,
        &["render", "history", "copy", "exposure"],
    );
    record_bool(
        store,
        "render.history.copy.volumetric_scattering_copied",
        frame_index,
        report.volumetric_scattering_copied,
        &["render", "history", "copy", "volumetric_scattering"],
    );
}

fn record_invalidation_reason(
    store: &mut DiagnosticStore,
    frame_index: u64,
    current_reason: Option<FrameHistoryInvalidationReason>,
    expected_reason: FrameHistoryInvalidationReason,
    path: &'static str,
    reason_tag: &'static str,
) {
    record_bool(
        store,
        path,
        frame_index,
        current_reason == Some(expected_reason),
        &["render", "history", "invalidation", reason_tag],
    );
}
