use crate::core::framework::render::{MotionVectorCameraStatus, RenderStats};

use super::{record_bool, record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
