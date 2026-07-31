use crate::core::framework::render::RenderSceneSnapshot;
use crate::core::math::UVec2;

use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};

use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_surface::SceneViewportSurface;

impl SceneRenderer {
    pub fn render(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport_size: impl Into<UVec2>,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame(&ViewportRenderFrame::from_snapshot(snapshot, viewport_size))
    }

    /// Renders to a native viewport surface without reading the final texture
    /// back through CPU memory. Call [`Self::render`] for screenshots or other
    /// consumers that require a [`ViewportFrame`].
    pub fn render_to_viewport_surface(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport_size: impl Into<UVec2>,
        surface: &mut SceneViewportSurface,
    ) -> Result<(), GraphicsError> {
        self.render_frame_to_viewport_surface(
            &ViewportRenderFrame::from_snapshot(snapshot, viewport_size),
            surface,
        )
    }
}
