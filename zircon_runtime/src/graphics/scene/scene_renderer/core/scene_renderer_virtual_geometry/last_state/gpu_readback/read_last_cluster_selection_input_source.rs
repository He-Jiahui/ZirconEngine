#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryClusterSelectionInputSource;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_cluster_selection_input_source(
        &self,
    ) -> RenderVirtualGeometryClusterSelectionInputSource {
        self.last_virtual_geometry_cluster_selection_input_source
    }
}
