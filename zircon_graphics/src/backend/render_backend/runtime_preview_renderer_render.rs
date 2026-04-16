use crate::scene::create_depth_texture;
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

use super::runtime_preview_renderer::RuntimePreviewRenderer;

impl RuntimePreviewRenderer {
    pub fn render(&mut self, frame: &EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        self.streamer.ensure_scene_resources(
            &self.backend.device,
            &self.backend.queue,
            &self.scene_renderer.texture_bind_group_layout,
            frame,
        )?;

        let output = self.surface_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth = create_depth_texture(&self.backend.device, self.surface_state.size);
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        self.scene_renderer.render_scene(
            &self.backend.device,
            &self.backend.queue,
            &self.streamer,
            frame,
            &view,
            &depth_view,
        )?;
        output.present();
        Ok(())
    }
}
