use crate::core::framework::render::RenderSceneSnapshot;
use crate::core::math::UVec2;

use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub fn render(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport_size: impl Into<UVec2>,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame(&ViewportRenderFrame::from_snapshot(snapshot, viewport_size))
    }
}
