use crate::core::framework::render::normalize_texture_max_anisotropy;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_texture_max_anisotropy(mut self, max_anisotropy: u8) -> Self {
        self.texture_max_anisotropy = normalize_texture_max_anisotropy(max_anisotropy);
        self
    }
}
