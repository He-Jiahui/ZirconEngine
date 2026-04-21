use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryVisBuffer64Source,
};
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_visbuffer64_source(
        &self,
    ) -> RenderVirtualGeometryVisBuffer64Source {
        self.last_virtual_geometry_visbuffer64_source
    }

    pub(crate) fn last_virtual_geometry_visbuffer64_entry_count(&self) -> u32 {
        self.last_virtual_geometry_visbuffer64_entry_count
    }

    pub(crate) fn last_virtual_geometry_hardware_rasterization_source(
        &self,
    ) -> RenderVirtualGeometryHardwareRasterizationSource {
        self.last_virtual_geometry_hardware_rasterization_source
    }

    pub(crate) fn last_virtual_geometry_hardware_rasterization_record_count(&self) -> u32 {
        self.last_virtual_geometry_hardware_rasterization_record_count
    }
}
