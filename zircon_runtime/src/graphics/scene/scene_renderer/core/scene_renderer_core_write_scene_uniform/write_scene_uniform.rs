use super::super::super::primitives::SceneUniform;
use super::super::scene_renderer_core::SceneRendererCore;
use crate::graphics::scene::resources::ResourceStreamer;

impl SceneRendererCore {
    pub(crate) fn write_scene_uniform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &crate::graphics::types::ViewportRenderFrame,
        reflection_probes_enabled: bool,
    ) {
        if let Some(environment) = frame.environment().skybox.source_cubemap_environment() {
            if self
                .scene_environment_cubemap
                .ensure_uploaded(device, queue, environment)
            {
                self.scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("zircon-scene-bind-group"),
                    layout: &self.scene_bind_group_layout,
                    entries: &self.scene_environment_cubemap.bind_group_entries(
                        &self.scene_uniform_buffer,
                        &self.scene_environment_brdf_lut,
                    ),
                });
            }
        }
        let _reflection_probe_upload_report = self.mesh_pipelines.reflection_probes.prepare(
            queue,
            streamer,
            frame,
            reflection_probes_enabled,
        );
        let scene_uniform = SceneUniform::from_frame(frame);
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniform),
        );
    }
}
