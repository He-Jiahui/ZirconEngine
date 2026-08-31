use std::collections::HashMap;

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
    let capacity = traversal_records
        .iter()
        .filter(|record| authored_child_record(record))
        .map(|record| available_authored_child_count(record, hierarchy_child_ids.len()))
        .fold(0_usize, usize::saturating_add);
    let mut child_work_items = Vec::with_capacity(capacity);
    for record in traversal_records
        .iter()
        .filter(|record| authored_child_record(record))
    {
        for child_offset in 0..record.child_count {
            let child_table_index = record.child_base.saturating_add(child_offset);
            let Ok(child_table_index_usize) = usize::try_from(child_table_index) else {
                continue;
            };
            let Some(child_node_id) = hierarchy_child_ids.get(child_table_index_usize).copied()
            else {
                continue;
            };

            child_work_items.push(VirtualGeometryNodeAndClusterCullChildWorkItem {
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
            });
        }
    }
    child_work_items
}

pub(super) fn build_node_and_cluster_cull_child_visit_records(
    child_work_items: &[VirtualGeometryNodeAndClusterCullChildWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    first_traversal_index: u32,
) -> Vec<VirtualGeometryNodeAndClusterCullTraversalRecord> {
    if child_work_items.is_empty() {
        return Vec::new();
    }

    let mut hierarchy_node_by_key = HashMap::with_capacity(hierarchy_nodes.len());
    for node in hierarchy_nodes {
        hierarchy_node_by_key
            .entry((node.instance_index, node.node_id))
            .or_insert(*node);
    }

    child_work_items
        .iter()
        .enumerate()
        .map(|(child_index, work_item)| {
            let node = hierarchy_node_by_key
                .get(&(work_item.instance_index, work_item.child_node_id))
                .copied();
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

fn authored_child_record(record: &VirtualGeometryNodeAndClusterCullTraversalRecord) -> bool {
    record.op == VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild
        && record.child_source
            == VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy
}

fn available_authored_child_count(
    record: &VirtualGeometryNodeAndClusterCullTraversalRecord,
    hierarchy_child_id_count: usize,
) -> usize {
    let Ok(child_base) = usize::try_from(record.child_base) else {
        return 0;
    };
    let Ok(child_count) = usize::try_from(record.child_count) else {
        return 0;
    };
    hierarchy_child_id_count
        .saturating_sub(child_base)
        .min(child_count)
}

#[cfg(test)]
mod allocation_tests;
