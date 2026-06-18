use crate::core::framework::render::ShaderQualityTier;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_shader_quality(mut self, shader_quality: ShaderQualityTier) -> Self {
        self.shader_quality = shader_quality;
        self
    }
}
