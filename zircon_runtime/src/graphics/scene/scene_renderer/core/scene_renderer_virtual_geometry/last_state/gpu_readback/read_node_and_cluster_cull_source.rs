#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryNodeAndClusterCullSource;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_node_and_cluster_cull_source(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullSource {
        self.advanced_plugin_outputs
            .virtual_geometry_node_and_cluster_cull_source
    }
}
