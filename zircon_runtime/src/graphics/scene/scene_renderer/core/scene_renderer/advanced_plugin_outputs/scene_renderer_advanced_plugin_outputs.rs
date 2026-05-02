use crate::core::framework::render::RenderPluginRendererOutputs;

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginOutputs {
    plugin_renderer_outputs: RenderPluginRendererOutputs,
}

impl SceneRendererAdvancedPluginOutputs {
    pub(super) fn plugin_renderer_outputs_ref(&self) -> &RenderPluginRendererOutputs {
        &self.plugin_renderer_outputs
    }

    pub(super) fn plugin_renderer_outputs_mut(&mut self) -> &mut RenderPluginRendererOutputs {
        &mut self.plugin_renderer_outputs
    }
}
