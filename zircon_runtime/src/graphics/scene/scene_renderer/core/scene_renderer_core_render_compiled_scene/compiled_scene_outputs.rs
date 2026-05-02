use super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCompiledSceneOutputs {
    advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
}

impl SceneRendererCompiledSceneOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    ) -> Self {
        Self {
            advanced_plugin_readbacks,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_parts(
        self,
    ) -> SceneRendererAdvancedPluginReadbacks {
        self.advanced_plugin_readbacks
    }
}
