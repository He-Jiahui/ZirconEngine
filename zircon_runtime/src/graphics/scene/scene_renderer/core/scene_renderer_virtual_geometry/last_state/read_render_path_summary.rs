use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_cluster_selection_input_source(
        &self,
    ) -> RenderVirtualGeometryClusterSelectionInputSource {
        self.last_virtual_geometry_cluster_selection_input_source
    }

    pub(crate) fn last_virtual_geometry_selected_cluster_source(
        &self,
    ) -> RenderVirtualGeometrySelectedClusterSource {
        self.last_virtual_geometry_selected_cluster_source
    }

    pub(crate) fn last_virtual_geometry_node_and_cluster_cull_source(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullSource {
        self.last_virtual_geometry_node_and_cluster_cull_source
    }

    pub(crate) fn last_virtual_geometry_node_and_cluster_cull_record_count(&self) -> u32 {
        self.last_virtual_geometry_node_and_cluster_cull_record_count
    }

    pub(crate) fn last_virtual_geometry_node_and_cluster_cull_instance_seed_count(&self) -> u32 {
        self.last_virtual_geometry_node_and_cluster_cull_instance_seed_count
    }

    pub(crate) fn last_virtual_geometry_selected_cluster_count(&self) -> u32 {
        self.last_virtual_geometry_selected_cluster_count
    }

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
