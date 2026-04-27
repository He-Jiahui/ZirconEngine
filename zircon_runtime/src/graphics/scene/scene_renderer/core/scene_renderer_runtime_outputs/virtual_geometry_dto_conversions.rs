use crate::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem as RenderNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem as RenderNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource as RenderNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp as RenderNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord as RenderNodeAndClusterCullTraversalRecord,
};
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};

impl From<VirtualGeometryNodeAndClusterCullClusterWorkItem>
    for RenderNodeAndClusterCullClusterWorkItem
{
    fn from(work_item: VirtualGeometryNodeAndClusterCullClusterWorkItem) -> Self {
        Self {
            instance_index: work_item.instance_index,
            entity: work_item.entity,
            cluster_array_index: work_item.cluster_array_index,
            hierarchy_node_id: work_item.hierarchy_node_id,
            cluster_budget: work_item.cluster_budget,
            page_budget: work_item.page_budget,
            forced_mip: work_item.forced_mip,
        }
    }
}

impl From<VirtualGeometryNodeAndClusterCullChildWorkItem>
    for RenderNodeAndClusterCullChildWorkItem
{
    fn from(work_item: VirtualGeometryNodeAndClusterCullChildWorkItem) -> Self {
        Self {
            instance_index: work_item.instance_index,
            entity: work_item.entity,
            parent_cluster_array_index: work_item.parent_cluster_array_index,
            parent_hierarchy_node_id: work_item.parent_hierarchy_node_id,
            child_node_id: work_item.child_node_id,
            child_table_index: work_item.child_table_index,
            traversal_index: work_item.traversal_index,
            cluster_budget: work_item.cluster_budget,
            page_budget: work_item.page_budget,
            forced_mip: work_item.forced_mip,
        }
    }
}

impl From<VirtualGeometryNodeAndClusterCullTraversalOp> for RenderNodeAndClusterCullTraversalOp {
    fn from(op: VirtualGeometryNodeAndClusterCullTraversalOp) -> Self {
        match op {
            VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode => Self::VisitNode,
            VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster => Self::StoreCluster,
            VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild => Self::EnqueueChild,
        }
    }
}

impl From<VirtualGeometryNodeAndClusterCullTraversalChildSource>
    for RenderNodeAndClusterCullTraversalChildSource
{
    fn from(source: VirtualGeometryNodeAndClusterCullTraversalChildSource) -> Self {
        match source {
            VirtualGeometryNodeAndClusterCullTraversalChildSource::None => Self::None,
            VirtualGeometryNodeAndClusterCullTraversalChildSource::CompatFixedFanout => {
                Self::CompatFixedFanout
            }
            VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy => {
                Self::AuthoredHierarchy
            }
        }
    }
}

impl From<VirtualGeometryNodeAndClusterCullTraversalRecord>
    for RenderNodeAndClusterCullTraversalRecord
{
    fn from(record: VirtualGeometryNodeAndClusterCullTraversalRecord) -> Self {
        Self {
            op: RenderNodeAndClusterCullTraversalOp::from(record.op),
            child_source: RenderNodeAndClusterCullTraversalChildSource::from(record.child_source),
            instance_index: record.instance_index,
            entity: record.entity,
            cluster_array_index: record.cluster_array_index,
            hierarchy_node_id: record.hierarchy_node_id,
            node_cluster_start: record.node_cluster_start,
            node_cluster_count: record.node_cluster_count,
            child_base: record.child_base,
            child_count: record.child_count,
            traversal_index: record.traversal_index,
            cluster_budget: record.cluster_budget,
            page_budget: record.page_budget,
            forced_mip: record.forced_mip,
        }
    }
}
