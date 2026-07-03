use super::super::scene_extract::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryInstance,
    RenderVirtualGeometryPageDependency,
};
use super::bvh_visualization::RenderVirtualGeometryBvhVisualizationInstance;
use super::cpu_reference::RenderVirtualGeometryCpuReferenceInstance;
use super::cull_input::RenderVirtualGeometryCullInputSnapshot;
use super::execution::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryPageRequestInspection, RenderVirtualGeometryResidentPageInspection,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySubmissionEntry,
    RenderVirtualGeometrySubmissionRecord, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBufferMark,
};
use super::node_and_cluster_cull::{
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
};
use super::page_payload::RenderVirtualGeometryPagePayload;
use super::sources::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryDebugSnapshot {
    pub instances: Vec<RenderVirtualGeometryInstance>,
    pub page_dependencies: Vec<RenderVirtualGeometryPageDependency>,
    pub resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
    pub debug: RenderVirtualGeometryDebugState,
    pub cull_input: RenderVirtualGeometryCullInputSnapshot,
    pub cluster_selection_input_source: RenderVirtualGeometryClusterSelectionInputSource,
    pub cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    pub bvh_visualization_instances: Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    pub visible_cluster_ids: Vec<u32>,
    pub selected_clusters: Vec<RenderVirtualGeometrySelectedCluster>,
    pub selected_clusters_source: RenderVirtualGeometrySelectedClusterSource,
    pub node_and_cluster_cull_source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub node_and_cluster_cull_record_count: u32,
    pub node_and_cluster_cull_instance_seeds:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub node_and_cluster_cull_instance_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub node_and_cluster_cull_cluster_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub node_and_cluster_cull_child_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub node_and_cluster_cull_traversal_records:
        Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub node_and_cluster_cull_hierarchy_child_ids: Vec<u32>,
    pub node_and_cluster_cull_page_request_ids: Vec<u32>,
    pub node_and_cluster_cull_dispatch_setup:
        Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub node_and_cluster_cull_launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub node_and_cluster_cull_global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub hardware_rasterization_records: Vec<RenderVirtualGeometryHardwareRasterizationRecord>,
    pub hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    pub visbuffer_debug_marks: Vec<RenderVirtualGeometryVisBufferMark>,
    pub visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    pub visbuffer64_clear_value: u64,
    pub visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub requested_pages: Vec<u32>,
    pub resident_pages: Vec<u32>,
    pub dirty_requested_pages: Vec<u32>,
    pub evictable_pages: Vec<u32>,
    pub resident_page_inspections: Vec<RenderVirtualGeometryResidentPageInspection>,
    pub pending_page_request_inspections: Vec<RenderVirtualGeometryPageRequestInspection>,
    pub available_page_slots: Vec<u32>,
    pub evictable_page_inspections: Vec<RenderVirtualGeometryResidentPageInspection>,
    pub leaf_clusters: Vec<RenderVirtualGeometryCluster>,
    pub execution_segment_count: u32,
    pub execution_page_count: u32,
    pub execution_resident_segment_count: u32,
    pub execution_pending_segment_count: u32,
    pub execution_missing_segment_count: u32,
    pub execution_repeated_draw_count: u32,
    pub execution_indirect_offsets: Vec<u64>,
    pub execution_segments: Vec<RenderVirtualGeometryExecutionSegment>,
    pub submission_order: Vec<RenderVirtualGeometrySubmissionEntry>,
    pub submission_records: Vec<RenderVirtualGeometrySubmissionRecord>,
}
