use crate::graphics::visibility::FrameVisibility;

impl super::viewport_render_frame::ViewportRenderFrame {
    pub(crate) fn with_frame_visibility(mut self, frame_visibility: FrameVisibility) -> Self {
        self.frame_visibility = Some(frame_visibility);
        self
    }
}
