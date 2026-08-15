use crate::graphics::backend::GpuPassTimer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_runtime_prepare_passes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        gpu_work_admitted: bool,
        gpu_pass_timer: Option<&mut GpuPassTimer>,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        self.advanced_plugin_resources
            .execute_runtime_prepare_passes_with_gpu_work_admission(
                device,
                queue,
                encoder,
                streamer,
                frame,
                gpu_work_admitted,
                gpu_pass_timer,
            )
    }
}
