use super::super::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBuffer64Entry,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryReadbackOutputs {
    pub page_table_entries: Vec<u32>,
    pub completed_page_assignments: Vec<RenderVirtualGeometryPageAssignmentRecord>,
    pub page_replacements: Vec<RenderVirtualGeometryPageReplacementRecord>,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    pub node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
}

impl RenderVirtualGeometryReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.page_table_entries.is_empty()
            && self.completed_page_assignments.is_empty()
            && self.page_replacements.is_empty()
            && self.selected_clusters.is_empty()
            && self.visbuffer64_entries.is_empty()
            && self.hardware_rasterization_records.is_empty()
            && self.node_cluster_cull.is_empty()
    }

    pub fn take_node_and_cluster_cull_page_request_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.node_cluster_cull.page_request_ids)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub launch_worklist_snapshots:
        Vec<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub page_request_ids: Vec<u32>,
}

impl RenderVirtualGeometryNodeClusterCullReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.traversal_records.is_empty()
            && self.child_work_items.is_empty()
            && self.cluster_work_items.is_empty()
            && self.launch_worklist_snapshots.is_empty()
            && self.page_request_ids.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageAssignmentRecord {
    pub page_id: u64,
    pub physical_slot: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageReplacementRecord {
    pub old_page_id: u64,
    pub new_page_id: u64,
    pub physical_slot: u32,
}
