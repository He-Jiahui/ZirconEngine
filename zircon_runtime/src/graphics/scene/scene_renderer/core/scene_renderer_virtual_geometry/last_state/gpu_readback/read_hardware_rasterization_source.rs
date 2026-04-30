#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryHardwareRasterizationSource;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_hardware_rasterization_source(
        &self,
    ) -> RenderVirtualGeometryHardwareRasterizationSource {
        self.advanced_plugin_outputs
            .virtual_geometry_hardware_rasterization_source()
    }
}
