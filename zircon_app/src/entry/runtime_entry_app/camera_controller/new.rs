use zircon_runtime::core::math::{UVec2, Vec3};

use super::runtime_camera_controller::RuntimeCameraController;

impl RuntimeCameraController {
    pub(in crate::entry::runtime_entry_app) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            orbit_target: Vec3::ZERO,
            drag: None,
        }
    }
}
