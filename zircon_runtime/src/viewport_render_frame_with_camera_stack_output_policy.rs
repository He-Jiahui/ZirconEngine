use super::{viewport_render_frame::ViewportRenderFrame, ViewportCameraStackOutputPolicy};

impl ViewportRenderFrame {
    pub(crate) fn with_camera_stack_output_policy(
        mut self,
        policy: ViewportCameraStackOutputPolicy,
    ) -> Self {
        self.camera_stack_output_policy = policy;
        self
    }
}
