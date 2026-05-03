use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _streamer: &ResourceStreamer,
        _frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        if !self.virtual_geometry_enabled() && !self.hybrid_gi_enabled() {
            return Ok(SceneRendererAdvancedPluginReadbacks::new());
        }

        Ok(SceneRendererAdvancedPluginReadbacks::new())
    }
}
