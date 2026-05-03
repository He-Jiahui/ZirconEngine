use super::super::super::graph_execution::RenderGraphExecutionRecord;
use super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererCompiledSceneOutputs {
    advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
    render_graph_execution: RenderGraphExecutionRecord,
}

impl SceneRendererCompiledSceneOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        advanced_plugin_readbacks: SceneRendererAdvancedPluginReadbacks,
        render_graph_execution: RenderGraphExecutionRecord,
    ) -> Self {
        Self {
            advanced_plugin_readbacks,
            render_graph_execution,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn into_parts(
        self,
    ) -> (SceneRendererAdvancedPluginReadbacks, RenderGraphExecutionRecord) {
        (self.advanced_plugin_readbacks, self.render_graph_execution)
    }
}
