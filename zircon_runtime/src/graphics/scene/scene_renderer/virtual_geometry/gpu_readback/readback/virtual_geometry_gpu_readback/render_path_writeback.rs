use super::VirtualGeometryGpuReadback;
use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};

impl VirtualGeometryGpuReadback {
    pub(crate) fn replace_render_path_readback(
        &mut self,
        hardware_rasterization_record_count: u32,
        hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
        selected_cluster_count: u32,
        selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
        selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
        visbuffer64_entry_count: u32,
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
        visbuffer64_clear_value: u64,
        visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    ) {
        self.hardware_rasterization_record_count = hardware_rasterization_record_count;
        self.hardware_rasterization_source = hardware_rasterization_source;
        self.selected_cluster_count = selected_cluster_count;
        self.selected_cluster_source = selected_cluster_source;
        self.selected_clusters = selected_clusters;
        self.visbuffer64_entry_count = visbuffer64_entry_count;
        self.visbuffer64_source = visbuffer64_source;
        self.visbuffer64_clear_value = visbuffer64_clear_value;
        self.visbuffer64_entries = visbuffer64_entries;
    }

    pub(crate) fn fill_missing_render_path_readback(
        &mut self,
        hardware_rasterization_record_count: u32,
        hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
        selected_cluster_count: u32,
        selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
        fallback_selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
        visbuffer64_entry_count: u32,
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
        visbuffer64_clear_value: u64,
        fallback_visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    ) {
        self.hardware_rasterization_record_count = hardware_rasterization_record_count;
        self.hardware_rasterization_source = hardware_rasterization_source;
        self.selected_cluster_count = selected_cluster_count;
        self.selected_cluster_source = selected_cluster_source;
        if self.selected_clusters.is_empty() {
            self.selected_clusters = fallback_selected_clusters;
        }
        self.visbuffer64_entry_count = visbuffer64_entry_count;
        self.visbuffer64_source = visbuffer64_source;
        self.visbuffer64_clear_value = visbuffer64_clear_value;
        if self.visbuffer64_entries.is_empty() {
            self.visbuffer64_entries = fallback_visbuffer64_entries;
        }
    }
}
