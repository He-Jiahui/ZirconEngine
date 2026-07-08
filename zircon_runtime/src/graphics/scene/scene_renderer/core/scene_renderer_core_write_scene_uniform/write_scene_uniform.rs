use super::super::super::primitives::SceneUniform;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn write_scene_uniform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &crate::graphics::types::ViewportRenderFrame,
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
        let scene_uniform = SceneUniform::from_frame(frame);
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniform),
        );
    }
}
