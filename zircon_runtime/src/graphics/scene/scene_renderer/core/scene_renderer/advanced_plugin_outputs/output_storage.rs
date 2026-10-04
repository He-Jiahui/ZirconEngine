use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::core::framework::render::RenderPluginRendererOutputs;

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn store_plugin_renderer_outputs(
        &mut self,
        outputs: RenderPluginRendererOutputs,
    ) {
        *self.plugin_renderer_outputs_mut() = outputs;
    }
}

#[cfg(test)]
#[path = "tests/output_storage.rs"]
mod tests;
