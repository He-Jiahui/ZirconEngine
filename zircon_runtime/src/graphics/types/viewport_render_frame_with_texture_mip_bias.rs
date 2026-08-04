use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_texture_mip_bias(mut self, mip_bias: u8) -> Self {
        self.texture_mip_bias = mip_bias;
        self
    }
}
