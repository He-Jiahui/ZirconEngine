use zircon_math::UVec2;
use zircon_scene::RenderSceneSnapshot;

use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame};

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub fn render(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport_size: impl Into<UVec2>,
    ) -> Result<ViewportFrame, GraphicsError> {
        self.render_frame(&EditorOrRuntimeFrame::from_snapshot(
            snapshot,
            viewport_size,
        ))
    }
}
