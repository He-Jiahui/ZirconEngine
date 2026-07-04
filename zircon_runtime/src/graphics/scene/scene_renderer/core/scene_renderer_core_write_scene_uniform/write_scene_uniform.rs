use super::super::super::primitives::SceneUniform;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn write_scene_uniform(
        &self,
        queue: &wgpu::Queue,
        frame: &crate::graphics::types::ViewportRenderFrame,
    ) {
        let scene_uniform = SceneUniform::from_frame(frame);
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniform),
        );
        queue.write_buffer(
            &self.scene_environment_sample_buffer,
            0,
            bytemuck::cast_slice(frame.environment().skybox.sampled_equirectangular_samples()),
        );
    }
}
