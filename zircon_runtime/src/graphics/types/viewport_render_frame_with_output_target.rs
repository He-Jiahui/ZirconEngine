use super::{viewport_render_frame::ViewportRenderFrame, ViewportRenderOutputTarget};

impl ViewportRenderFrame {
    pub(crate) fn with_output_target(mut self, output_target: ViewportRenderOutputTarget) -> Self {
        self.output_target = output_target;
        self
    }
}
