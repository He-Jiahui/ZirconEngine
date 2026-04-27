#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryHardwareRasterizationSource;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn read_last_virtual_geometry_gpu_readback_hardware_rasterization_source(
        &self,
    ) -> Option<RenderVirtualGeometryHardwareRasterizationSource> {
        self.advanced_plugin_outputs
            .virtual_geometry_gpu_readback()
            .map(|readback| readback.hardware_rasterization_source)
    }
}
