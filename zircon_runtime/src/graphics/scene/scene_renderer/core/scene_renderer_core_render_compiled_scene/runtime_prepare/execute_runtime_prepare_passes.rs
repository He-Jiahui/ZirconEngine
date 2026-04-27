use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_runtime_prepare_passes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        self.advanced_plugin_resources
            .execute_runtime_prepare_passes(device, queue, encoder, streamer, frame)
    }
}
