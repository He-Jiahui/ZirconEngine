use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryGpuReadback {
    pub(crate) page_table_entries: Vec<(u32, u32)>,
    pub(crate) completed_page_ids: Vec<u32>,
    pub(crate) completed_page_assignments: Vec<(u32, u32)>,
    pub(crate) completed_page_replacements: Vec<(u32, u32)>,
    pub(crate) hardware_rasterization_record_count: u32,
    pub(crate) hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub(crate) selected_cluster_count: u32,
    pub(crate) selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
    pub(crate) selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub(crate) visbuffer64_entry_count: u32,
    pub(crate) visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub(crate) visbuffer64_clear_value: u64,
    pub(crate) visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
}
