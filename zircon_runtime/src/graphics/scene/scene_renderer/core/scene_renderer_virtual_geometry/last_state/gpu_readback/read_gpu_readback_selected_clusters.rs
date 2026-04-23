#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometrySelectedCluster;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_gpu_readback_selected_clusters(
        &self,
    ) -> Option<Vec<RenderVirtualGeometrySelectedCluster>> {
        self.last_virtual_geometry_gpu_readback
            .as_ref()
            .map(|readback| readback.selected_clusters.clone())
    }
}
