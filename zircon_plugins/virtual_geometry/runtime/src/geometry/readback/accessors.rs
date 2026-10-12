use super::VirtualGeometryGpuReadback;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};

#[cfg_attr(not(test), allow(dead_code))]
impl VirtualGeometryGpuReadback {
    pub(crate) fn page_table_entries(&self) -> &[(u32, u32)] {
        &self.page_table_entries
    }

    pub(crate) fn completed_page_ids(&self) -> &[u32] {
        &self.completed_page_ids
    }

    pub(crate) fn completed_page_assignments(&self) -> &[(u32, u32)] {
        &self.completed_page_assignments
    }

    pub(crate) fn completed_page_replacements(&self) -> &[(u32, u32)] {
        &self.completed_page_replacements
    }

    pub(crate) fn hardware_rasterization_record_count(&self) -> u32 {
        self.hardware_rasterization_record_count
    }

    pub(crate) fn hardware_rasterization_source(
        &self,
    ) -> RenderVirtualGeometryHardwareRasterizationSource {
        self.hardware_rasterization_source
    }

    pub(crate) fn hardware_rasterization_records(
        &self,
    ) -> &[RenderVirtualGeometryHardwareRasterizationRecord] {
        &self.hardware_rasterization_records
    }

    pub(crate) fn selected_cluster_count(&self) -> u32 {
        self.selected_cluster_count
    }

    pub(crate) fn selected_cluster_source(&self) -> RenderVirtualGeometrySelectedClusterSource {
        self.selected_cluster_source
    }

    pub(crate) fn selected_clusters(&self) -> &[RenderVirtualGeometrySelectedCluster] {
        &self.selected_clusters
    }

    pub(crate) fn visbuffer64_entry_count(&self) -> u32 {
        self.visbuffer64_entry_count
    }

    pub(crate) fn visbuffer64_source(&self) -> RenderVirtualGeometryVisBuffer64Source {
        self.visbuffer64_source
    }

    pub(crate) fn visbuffer64_clear_value(&self) -> u64 {
        self.visbuffer64_clear_value
    }

    pub(crate) fn visbuffer64_entries(&self) -> &[RenderVirtualGeometryVisBuffer64Entry] {
        &self.visbuffer64_entries
    }

    pub(crate) fn visbuffer64_packed_words(&self) -> Vec<u64> {
        self.visbuffer64_entries
            .iter()
            .map(|entry| entry.packed_value)
            .collect()
    }

    pub(crate) fn node_cluster_cull(&self) -> &RenderVirtualGeometryNodeClusterCullReadbackOutputs {
        &self.node_cluster_cull
    }
}
