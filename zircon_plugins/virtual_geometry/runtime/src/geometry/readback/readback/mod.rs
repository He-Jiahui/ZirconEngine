use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};

#[path = "../accessors.rs"]
mod accessors;
#[path = "../completion.rs"]
mod completion;
#[path = "../../../node_cluster_cull_writeback.rs"]
mod node_cluster_cull_writeback;
#[path = "../../render_path_writeback.rs"]
mod render_path_writeback;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VirtualGeometryGpuReadback {
    page_table_entries: Vec<(u32, u32)>,
    completed_page_ids: Vec<u32>,
    completed_page_assignments: Vec<(u32, u32)>,
    completed_page_replacements: Vec<(u32, u32)>,
    hardware_rasterization_record_count: u32,
    hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    selected_cluster_count: u32,
    selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
    selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    visbuffer64_entry_count: u32,
    visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    visbuffer64_clear_value: u64,
    visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
}
