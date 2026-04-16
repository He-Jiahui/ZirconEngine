use zircon_math::Vec2;

use super::runtime_preview_renderer::RuntimePreviewRenderer;

impl RuntimePreviewRenderer {
    pub fn viewport_center(&self) -> Vec2 {
        Vec2::new(
            self.surface_state.size.x as f32 * 0.5,
            self.surface_state.size.y as f32 * 0.5,
        )
    }
}
