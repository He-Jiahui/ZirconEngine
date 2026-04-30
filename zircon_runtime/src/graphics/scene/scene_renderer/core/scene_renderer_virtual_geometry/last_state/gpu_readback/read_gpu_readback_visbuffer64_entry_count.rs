use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn read_last_virtual_geometry_gpu_readback_visbuffer64_entry_count(
        &self,
    ) -> Option<u32> {
        self.advanced_plugin_outputs
            .virtual_geometry_gpu_readback()
            .map(|readback| readback.visbuffer64_entry_count())
    }
}
