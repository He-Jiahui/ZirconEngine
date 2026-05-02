use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::core::framework::render::RenderPluginRendererOutputs;

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn has_virtual_geometry_gpu_readback(
        &self,
    ) -> bool {
        !self
            .plugin_renderer_outputs_ref()
            .virtual_geometry
            .page_table_entries
            .is_empty()
            || !self
                .plugin_renderer_outputs_ref()
                .virtual_geometry
                .selected_clusters
                .is_empty()
            || !self
                .plugin_renderer_outputs_ref()
                .virtual_geometry
                .visbuffer64_entries
                .is_empty()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn plugin_renderer_outputs(
        &self,
    ) -> &RenderPluginRendererOutputs {
        self.plugin_renderer_outputs_ref()
    }
}
