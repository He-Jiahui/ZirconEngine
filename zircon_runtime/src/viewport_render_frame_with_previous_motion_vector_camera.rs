use crate::core::framework::render::ViewportCameraSnapshot;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_previous_motion_vector_camera(
        mut self,
        camera: Option<ViewportCameraSnapshot>,
    ) -> Self {
        self.previous_motion_vector_camera = camera;
        self
    }
}
