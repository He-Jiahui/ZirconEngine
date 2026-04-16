use crate::backend::{read_texture_rgba, OffscreenTarget, RenderBackend};
use crate::scene::{ResourceStreamer, SceneRendererCore};
use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrame};

use super::offscreen_renderer::OffscreenRenderer;

impl OffscreenRenderer {
    pub(in crate::service) fn render(
        &mut self,
        backend: &mut RenderBackend,
        scene_renderer: &mut SceneRendererCore,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        generation: u64,
    ) -> Result<ViewportFrame, GraphicsError> {
        let size =
            zircon_math::UVec2::new(frame.viewport.size.x.max(1), frame.viewport.size.y.max(1));
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.size != size)
        {
            self.target = Some(OffscreenTarget::new(&backend.device, size));
        }
        let target = self.target.as_ref().unwrap();

        scene_renderer.render_scene(
            &backend.device,
            &backend.queue,
            streamer,
            frame,
            &target.final_color_view,
            &target.depth_view,
        )?;
        let rgba = read_texture_rgba(
            &backend.device,
            &backend.queue,
            &target.final_color,
            target.size,
        )?;

        Ok(ViewportFrame {
            width: target.size.x,
            height: target.size.y,
            rgba,
            generation,
        })
    }
}
