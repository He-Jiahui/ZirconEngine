use zircon_math::UVec2;

use super::runtime_preview_renderer::RuntimePreviewRenderer;

impl RuntimePreviewRenderer {
    pub fn resize(&mut self, size: UVec2) {
        self.surface_state.resize(&self.backend.device, size);
    }
}
