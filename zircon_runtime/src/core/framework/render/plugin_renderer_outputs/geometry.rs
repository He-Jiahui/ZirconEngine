use super::super::{
    RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBuffer64Entry,
};

/// Inspection delivered only by the completed native terminal staging map.
/// Input and policy sections are uploaded-input evidence; instance work items are compute output.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryNativeReadbackInspection {
    pub cull_input: super::super::RenderVirtualGeometryCullInputSnapshot,
    pub startup: Option<RenderVirtualGeometryNativeStartupReadback>,
    pub selected_cluster_source: super::super::RenderVirtualGeometrySelectedClusterSource,
    pub hardware_rasterization_source:
        super::super::RenderVirtualGeometryHardwareRasterizationSource,
    pub visbuffer64_source: super::super::RenderVirtualGeometryVisBuffer64Source,
}

/// Fixed-layout startup buffers copied from the actual current-frame native resources.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderVirtualGeometryNativeStartupReadback {
    pub global_state:
        Option<super::super::RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub dispatch_setup:
        Option<super::super::RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub launch_worklist:
        Option<super::super::RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub instance_seeds: Vec<super::super::RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub instance_work_items:
        Vec<super::super::RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub cluster_work_items:
        Vec<super::super::RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub hierarchy_child_ids: Vec<u32>,
    pub child_work_items: Vec<super::super::RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub traversal_records:
        Vec<super::super::RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub page_request_ids: Vec<u32>,
}

/// 虚拟几何 provider 的页驻留、选择和剔除读回；提交端将请求 ID 与 GPU 完成信息分路处理。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryReadbackOutputs {
    pub page_table_entries: Vec<u32>,
    pub completed_page_assignments: Vec<RenderVirtualGeometryPageAssignmentRecord>,
    pub page_replacements: Vec<RenderVirtualGeometryPageReplacementRecord>,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    /// Only completed native terminal output; Some(empty) is a real clear completion.
    pub physical_submission_segments:
        Option<Vec<super::super::RenderVirtualGeometryExecutionSegment>>,
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
            && self.physical_submission_segments.is_none()
    }

    /// 在构造 GPU completion 前转移页请求，供流送反馈单独处理。
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
