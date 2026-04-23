use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn read_last_virtual_geometry_gpu_readback_selected_cluster_count(
        &self,
    ) -> Option<u32> {
        self.last_virtual_geometry_gpu_readback
            .as_ref()
            .map(|readback| readback.selected_cluster_count)
    }
}
