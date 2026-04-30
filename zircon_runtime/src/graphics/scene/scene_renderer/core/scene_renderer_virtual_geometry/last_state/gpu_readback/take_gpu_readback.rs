use crate::graphics::scene::scene_renderer::core::SceneRenderer;
use crate::graphics::scene::scene_renderer::VirtualGeometryGpuReadback;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn take_last_virtual_geometry_gpu_readback(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadback> {
        self.advanced_plugin_outputs
            .take_virtual_geometry_gpu_readback()
    }
}
