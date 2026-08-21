use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::graphics::backend::ViewportSurface;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(in crate::graphics) fn create_framework_viewport_surface(
        &self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<ViewportSurface, GraphicsError> {
        self.backend.create_viewport_surface(descriptor)
    }

    pub(crate) fn present_frame_direct(
        &mut self,
        frame: &ViewportRenderFrame,
        surface: &mut ViewportSurface,
    ) -> Result<u64, GraphicsError> {
        self.render_frame_to_offscreen_target(frame)?;
        let target = self.target.as_ref().expect("offscreen target");
        surface.present_texture(
            &self.backend.device,
            &self.backend.queue,
            &target.final_color_view,
        )?;
        Ok(self.generation)
    }
}
