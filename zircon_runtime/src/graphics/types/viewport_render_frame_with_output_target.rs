use super::{ViewportRenderOutputTarget, viewport_render_frame::ViewportRenderFrame};

impl ViewportRenderFrame {
    pub(crate) fn with_output_target(mut self, output_target: ViewportRenderOutputTarget) -> Self {
        self.output_target = output_target;
        self
    }
}
