use crate::types::{EditorOrRuntimeFrame, GraphicsError};

use super::super::super::resources::ResourceStreamer;
use super::super::mesh::build_mesh_draws;
use super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) -> Result<(), GraphicsError> {
        self.write_scene_uniform(queue, frame);
        let mesh_draws = build_mesh_draws(
            device,
            &self.model_bind_group_layout,
            streamer,
            frame,
            false,
        );
        let prepared_overlays = self.overlay_renderer.prepare_buffers(
            device,
            queue,
            &self.texture_bind_group_layout,
            streamer,
            frame,
        )?;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-encoder"),
        });
        self.overlay_renderer.record(
            &mut encoder,
            device,
            color_view,
            depth_view,
            &self.scene_bind_group,
            &mesh_draws,
            &mut self.mesh_pipelines,
            streamer,
            frame,
            &prepared_overlays,
        );
        queue.submit([encoder.finish()]);
        Ok(())
    }
}
