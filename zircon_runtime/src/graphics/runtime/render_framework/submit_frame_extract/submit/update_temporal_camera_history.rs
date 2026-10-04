use crate::graphics::ViewportRenderFrame;

use super::super::super::viewport_record::{ViewportCameraHistoryKey, ViewportRecord};

pub(super) fn update_temporal_camera_history_after_success(
    record: &mut ViewportRecord,
    frame: &ViewportRenderFrame,
    camera_history_key: &ViewportCameraHistoryKey,
    advance_temporal_frame_index: bool,
) {
    let mut camera = frame.effective_camera();
    camera.temporal_jitter = Default::default();
    record.replace_motion_vector_camera(camera_history_key, camera);
    if advance_temporal_frame_index {
        record.advance_temporal_frame_index();
    }
}

#[cfg(test)]
#[path = "tests/update_temporal_camera_history.rs"]
mod tests;
