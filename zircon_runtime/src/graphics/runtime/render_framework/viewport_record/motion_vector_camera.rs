use crate::core::framework::render::ViewportCameraSnapshot;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn motion_vector_camera(
        &self,
    ) -> Option<&ViewportCameraSnapshot> {
        self.motion_vector_camera.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_motion_vector_camera(
        &mut self,
        camera: ViewportCameraSnapshot,
    ) {
        self.motion_vector_camera = Some(camera);
    }
}
