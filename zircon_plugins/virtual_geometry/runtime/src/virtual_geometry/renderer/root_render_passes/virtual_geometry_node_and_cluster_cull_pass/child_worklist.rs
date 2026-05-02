use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};
use zircon_runtime::core::framework::render::RenderVirtualGeometryHierarchyNode;

pub(super) fn build_node_and_cluster_cull_child_work_items(
    traversal_records: &[VirtualGeometryNodeAndClusterCullTraversalRecord],
    hierarchy_child_ids: &[u32],
) -> Vec<VirtualGeometryNodeAndClusterCullChildWorkItem> {
    traversal_records
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild
                && record.child_source
                    == VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy
        })
        .flat_map(|record| {
            (0..record.child_count).filter_map(move |child_offset| {
                let child_table_index = record.child_base.saturating_add(child_offset);
                let child_table_index_usize = usize::try_from(child_table_index).ok()?;
                let child_node_id = *hierarchy_child_ids.get(child_table_index_usize)?;

                Some(VirtualGeometryNodeAndClusterCullChildWorkItem {
                    instance_index: record.instance_index,
                    entity: record.entity,
                    parent_cluster_array_index: record.cluster_array_index,
                    parent_hierarchy_node_id: record.hierarchy_node_id,
                    child_node_id,
                    child_table_index,
                    traversal_index: record.traversal_index,
                    cluster_budget: record.cluster_budget,
                    page_budget: record.page_budget,
                    forced_mip: record.forced_mip,
                })
            })
        })
        .collect()
}

pub(super) fn build_node_and_cluster_cull_child_visit_records(
    child_work_items: &[VirtualGeometryNodeAndClusterCullChildWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    first_traversal_index: u32,
) -> Vec<VirtualGeometryNodeAndClusterCullTraversalRecord> {
    child_work_items
        .iter()
        .enumerate()
        .map(|(child_index, work_item)| {
            let node = hierarchy_node_for_child_work_item(*work_item, hierarchy_nodes);
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: work_item.instance_index,
                entity: work_item.entity,
                cluster_array_index: work_item.parent_cluster_array_index,
                hierarchy_node_id: Some(work_item.child_node_id),
                node_cluster_start: node.map(|node| node.cluster_start).unwrap_or(0),
                node_cluster_count: node.map(|node| node.cluster_count).unwrap_or(0),
                child_base: 0,
                child_count: 0,
                traversal_index: first_traversal_index
                    .saturating_add(u32::try_from(child_index).unwrap_or(u32::MAX)),
                cluster_budget: work_item.cluster_budget,
                page_budget: work_item.page_budget,
                forced_mip: work_item.forced_mip,
            }
        })
        .collect()
}

fn hierarchy_node_for_child_work_item(
    work_item: VirtualGeometryNodeAndClusterCullChildWorkItem,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> Option<RenderVirtualGeometryHierarchyNode> {
    hierarchy_nodes.iter().copied().find(|node| {
        node.instance_index == work_item.instance_index && node.node_id == work_item.child_node_id
    })
}
