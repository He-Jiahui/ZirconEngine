use crate::graphics::scene::scene_renderer::core::scene_renderer::SceneRendererAdvancedPluginOutputs;
use crate::graphics::types::GraphicsError;

use super::scene_renderer_advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn collect_into_outputs(
        self,
        outputs: &mut SceneRendererAdvancedPluginOutputs,
    ) -> Result<(), GraphicsError> {
        self.collect_neutral_outputs_into(outputs);
        Ok(())
    }

    fn collect_neutral_outputs_into(mut self, outputs: &mut SceneRendererAdvancedPluginOutputs) {
        outputs.store_plugin_renderer_outputs(std::mem::take(&mut self.outputs));
    }
}

#[cfg(test)]
#[path = "tests/collect_into_outputs.rs"]
mod tests;
