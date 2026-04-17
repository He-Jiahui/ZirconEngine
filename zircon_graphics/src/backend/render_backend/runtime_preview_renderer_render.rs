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

        let mut needs_reconfigure = false;
        let output = match self.surface_state.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) => output,
            wgpu::CurrentSurfaceTexture::Suboptimal(output) => {
                needs_reconfigure = true;
                output
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface_state
                    .surface
                    .configure(&self.backend.device, &self.surface_state.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(GraphicsError::SurfaceStatus("lost"));
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(GraphicsError::SurfaceStatus("validation"));
            }
        };
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
        if needs_reconfigure {
            self.surface_state
                .surface
                .configure(&self.backend.device, &self.surface_state.config);
        }
        Ok(())
    }
}
