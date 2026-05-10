use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::graphics::backend::ViewportSurface;
use crate::graphics::types::GraphicsError;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn create_viewport_surface(
        &self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<ViewportSurface, GraphicsError> {
        self.backend.create_viewport_surface(descriptor)
    }
}
