use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::graphics::backend::ViewportSurface;
use crate::graphics::types::GraphicsError;

use super::scene_renderer::SceneRenderer;

/// A native presentation surface owned by a [`SceneRenderer`].
///
/// Rendering through this surface keeps the scene color texture on the GPU. Use
/// [`SceneRenderer::render`] when the caller needs CPU RGBA pixels for capture
/// or image encoding.
pub struct SceneViewportSurface {
    pub(in crate::graphics::scene::scene_renderer::core) inner: ViewportSurface,
}

impl SceneViewportSurface {
    pub fn size(&self) -> crate::core::math::UVec2 {
        self.inner.size()
    }

    /// Transfers the scene-created surface into the framework record, which is
    /// the sole owner for its bound lifetime.
    pub(in crate::graphics) fn into_backend_surface(self) -> ViewportSurface {
        self.inner
    }
}

impl SceneRenderer {
    pub fn create_viewport_surface(
        &self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<SceneViewportSurface, GraphicsError> {
        self.backend
            .create_viewport_surface(descriptor)
            .map(|inner| SceneViewportSurface { inner })
    }
}
