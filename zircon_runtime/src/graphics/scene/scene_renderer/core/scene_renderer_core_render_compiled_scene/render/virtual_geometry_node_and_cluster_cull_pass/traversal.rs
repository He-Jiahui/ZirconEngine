use crate::core::framework::render::RenderVirtualGeometryHierarchyNode;
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};

const NODE_AND_CLUSTER_CULL_CHILD_FANOUT: u32 = 4;

pub(super) fn build_node_and_cluster_cull_traversal_records(
    cluster_work_items: &[VirtualGeometryNodeAndClusterCullClusterWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> Vec<VirtualGeometryNodeAndClusterCullTraversalRecord> {
    let mut traversal_records = Vec::new();
    let mut traversal_index = 0u32;
    let mut stored_cluster_count = 0u32;

    for work_item in cluster_work_items {
        traversal_records.push(node_and_cluster_cull_traversal_record(
            *work_item,
            VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
            traversal_index,
            hierarchy_nodes,
        ));
        traversal_index = traversal_index.saturating_add(1);

        if stored_cluster_count < work_item.cluster_budget {
            traversal_records.push(node_and_cluster_cull_traversal_record(
                *work_item,
                VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                traversal_index,
                hierarchy_nodes,
            ));
            traversal_index = traversal_index.saturating_add(1);
            stored_cluster_count = stored_cluster_count.saturating_add(1);
        } else {
            traversal_records.push(node_and_cluster_cull_traversal_record(
                *work_item,
                VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                traversal_index,
                hierarchy_nodes,
            ));
            traversal_index = traversal_index.saturating_add(1);
        }
    }

    traversal_records
}

pub(super) fn next_node_and_cluster_cull_traversal_index(
    traversal_records: &[VirtualGeometryNodeAndClusterCullTraversalRecord],
) -> u32 {
    traversal_records
        .iter()
        .map(|record| record.traversal_index)
        .max()
        .map(|index| index.saturating_add(1))
        .unwrap_or(0)
}

fn node_and_cluster_cull_traversal_record(
    work_item: VirtualGeometryNodeAndClusterCullClusterWorkItem,
    op: VirtualGeometryNodeAndClusterCullTraversalOp,
    traversal_index: u32,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> VirtualGeometryNodeAndClusterCullTraversalRecord {
    let (child_source, child_base, child_count) = match op {
        VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild => {
            authored_hierarchy_child_range(work_item, hierarchy_nodes).unwrap_or((
                VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
                work_item
                    .cluster_array_index
                    .saturating_mul(NODE_AND_CLUSTER_CULL_CHILD_FANOUT),
                NODE_AND_CLUSTER_CULL_CHILD_FANOUT,
            ))
        }
        VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode
        | VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster => (
            VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
            0,
            0,
        ),
    };

    VirtualGeometryNodeAndClusterCullTraversalRecord {
        op,
        child_source,
        instance_index: work_item.instance_index,
        entity: work_item.entity,
        cluster_array_index: work_item.cluster_array_index,
        hierarchy_node_id: work_item.hierarchy_node_id,
        node_cluster_start: 0,
        node_cluster_count: 0,
        child_base,
        child_count,
        traversal_index,
        cluster_budget: work_item.cluster_budget,
        page_budget: work_item.page_budget,
        forced_mip: work_item.forced_mip,
    }
}

fn authored_hierarchy_child_range(
    work_item: VirtualGeometryNodeAndClusterCullClusterWorkItem,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> Option<(
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    u32,
    u32,
)> {
    let hierarchy_node_id = work_item.hierarchy_node_id?;
    let node = hierarchy_nodes.iter().find(|node| {
        node.instance_index == work_item.instance_index && node.node_id == hierarchy_node_id
    })?;
    (node.child_count > 0).then_some((
        VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        node.child_base,
        node.child_count,
    ))
}
