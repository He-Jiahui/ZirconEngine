#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometrySelectedClusterSource;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_selected_cluster_source(
        &self,
    ) -> RenderVirtualGeometrySelectedClusterSource {
        self.last_virtual_geometry_selected_cluster_source
    }
}
