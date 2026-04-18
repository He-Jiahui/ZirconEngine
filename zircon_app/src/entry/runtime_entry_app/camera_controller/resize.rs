use zircon_math::UVec2;

use super::runtime_camera_controller::RuntimeCameraController;

impl RuntimeCameraController {
    pub(in crate::entry::runtime_entry_app) fn resize(&mut self, size: UVec2) {
        self.viewport_size = UVec2::new(size.x.max(1), size.y.max(1));
    }
}
