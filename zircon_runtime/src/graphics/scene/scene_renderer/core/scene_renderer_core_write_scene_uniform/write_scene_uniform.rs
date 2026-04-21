use crate::core::math::Real;

use super::super::super::primitives::SceneUniform;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn write_scene_uniform(
        &self,
        queue: &wgpu::Queue,
        frame: &crate::graphics::types::ViewportRenderFrame,
    ) {
        let aspect = frame.viewport_size.x as Real / frame.viewport_size.y.max(1) as Real;
        let scene_uniform = SceneUniform::from_frame(frame, aspect);
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniform),
        );
    }
}
