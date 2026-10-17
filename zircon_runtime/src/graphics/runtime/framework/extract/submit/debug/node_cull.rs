//! Projects the provider-owned CPU preparation; never recomputes policy for diagnostics.
use super::super::super::frame_submission_context::FrameSubmissionContext;
use crate::core::framework::render::{
    RenderFrameExtract, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
};
pub(super) struct NodeAndClusterCullSnapshot {
    pub(super) source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub(super) record_count: u32,
    pub(super) global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(super) dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub(super) launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub(super) instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub(super) instance_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub(super) cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub(super) child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub(super) traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub(super) page_request_ids: Vec<u32>,
}

pub(super) fn build_node_and_cluster_cull_snapshot(
    _frame_extract: &RenderFrameExtract,
    context: &FrameSubmissionContext,
    _cull_input: RenderVirtualGeometryCullInputSnapshot,
) -> NodeAndClusterCullSnapshot {
    let frame = context.virtual_geometry_prepared_cpu_frame();
    let unavailable =
        crate::core::framework::render::RenderVirtualGeometryPreparedCpuFrame::unavailable();
    let inspection = &frame.as_deref().unwrap_or(&unavailable).inspection;
    NodeAndClusterCullSnapshot {
        source: inspection.source,
        record_count: inspection.record_count,
        global_state: inspection.global_state.clone(),
        dispatch_setup: inspection.dispatch_setup,
        launch_worklist: inspection.launch_worklist.clone(),
        instance_seeds: inspection.instance_seeds.clone(),
        instance_work_items: inspection.instance_work_items.clone(),
        cluster_work_items: inspection.cluster_work_items.clone(),
        child_work_items: inspection.child_work_items.clone(),
        traversal_records: inspection.traversal_records.clone(),
        page_request_ids: inspection.page_request_ids.clone(),
    }
}

#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryHierarchyNode;
#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
fn index_hierarchy_nodes(
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> HashMap<u32, &RenderVirtualGeometryHierarchyNode> {
    let mut nodes_by_id = HashMap::with_capacity(hierarchy_nodes.len());
    for node in hierarchy_nodes {
        nodes_by_id.entry(node.node_id).or_insert(node);
    }
    nodes_by_id
}

#[cfg(test)]
#[path = "../../../../extract/submit/debug/tests/node_cull.rs"]
mod tests;
