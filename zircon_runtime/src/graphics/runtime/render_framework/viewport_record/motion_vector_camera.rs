//! 运动矢量相机历史按相机键推进，仅成功提交后的上一帧矩阵可供时间后处理复用。
use crate::core::framework::render::ViewportCameraSnapshot;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn motion_vector_camera(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&ViewportCameraSnapshot> {
        self.motion_vector_cameras.get(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_motion_vector_camera(
        &mut self,
        key: &ViewportCameraHistoryKey,
        camera: ViewportCameraSnapshot,
    ) {
        if let Some(previous) = self.motion_vector_cameras.get_mut(key) {
            *previous = camera;
        } else {
            self.motion_vector_cameras.insert(key.clone(), camera);
        }
    }
}

#[cfg(test)]
#[path = "tests/motion_vector_camera.rs"]
mod tests;
