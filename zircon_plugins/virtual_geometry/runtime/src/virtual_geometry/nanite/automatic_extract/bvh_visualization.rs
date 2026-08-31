use std::collections::{HashMap, HashSet};

use zircon_runtime::asset::{VirtualGeometryAsset, VirtualGeometryHierarchyNodeAsset};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryBvhVisualizationNode,
};

use super::{VirtualGeometryCpuReferenceFrame, VirtualGeometryCpuReferenceLeafCluster};

pub(super) fn append_bvh_visualization_instance_if_enabled(
    output: &mut Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    visualize_bvh: bool,
    instance_index: u32,
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) {
    if !visualize_bvh {
        return;
    }
    output.push(render_bvh_visualization_instance(
        instance_index,
        asset,
        frame,
    ));
}

fn render_bvh_visualization_instance(
    instance_index: u32,
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> RenderVirtualGeometryBvhVisualizationInstance {
    let mut nodes_by_id = HashMap::with_capacity(asset.hierarchy_buffer.len());
    for node in &asset.hierarchy_buffer {
        nodes_by_id.insert(node.node_id, node);
    }
    let mut visited_nodes = HashMap::with_capacity(frame.visited_nodes().len());
    for visit in frame.visited_nodes() {
        visited_nodes.insert(visit.node_id(), visit);
    }
    let mut selected_cluster_ids = HashSet::with_capacity(frame.selected_clusters().len());
    selected_cluster_ids.extend(
        frame
            .selected_clusters()
            .iter()
            .map(VirtualGeometryCpuReferenceLeafCluster::cluster_id),
    );
    let subtree_leaf_clusters = subtree_leaf_cluster_index(&nodes_by_id, frame.leaf_clusters());

    let mut nodes = Vec::with_capacity(frame.visited_nodes().len());
    for visit in frame.visited_nodes() {
        let Some(node) = nodes_by_id.get(&visit.node_id()).copied() else {
            continue;
        };
        let subtree_clusters = subtree_leaf_clusters
            .get(&node.node_id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut selected_subtree_cluster_ids = Vec::with_capacity(subtree_clusters.len());
        let mut resident_cluster_ids = Vec::with_capacity(subtree_clusters.len());
        for cluster in subtree_clusters {
            if selected_cluster_ids.contains(&cluster.cluster_id()) {
                selected_subtree_cluster_ids.push(cluster.cluster_id());
            }
            if cluster.loaded() {
                resident_cluster_ids.push(cluster.cluster_id());
            }
        }
        nodes.push(RenderVirtualGeometryBvhVisualizationNode {
            node_id: node.node_id,
            parent_node_id: node.parent_node_id,
            child_node_ids: node.child_node_ids.clone(),
            depth: visited_nodes
                .get(&node.node_id)
                .map(|visited_node| visited_node.depth())
                .unwrap_or_default(),
            page_id: node.page_id,
            mip_level: node.mip_level,
            is_leaf: node.child_node_ids.is_empty(),
            cluster_ids: visit.cluster_ids().to_vec(),
            selected_cluster_ids: selected_subtree_cluster_ids,
            resident_cluster_ids,
            bounds_center: node.bounds_center,
            bounds_radius: node.bounds_radius,
            screen_space_error: node.screen_space_error,
        });
    }

    RenderVirtualGeometryBvhVisualizationInstance {
        instance_index,
        entity: frame.entity(),
        mesh_name: frame.mesh_name().map(str::to_owned),
        source_hint: frame.source_hint().map(str::to_owned),
        nodes,
    }
}

fn subtree_leaf_cluster_index<'a>(
    nodes_by_id: &HashMap<u32, &VirtualGeometryHierarchyNodeAsset>,
    leaf_clusters: &'a [VirtualGeometryCpuReferenceLeafCluster],
) -> HashMap<u32, Vec<&'a VirtualGeometryCpuReferenceLeafCluster>> {
    let mut parent_node_ids_by_child = HashMap::<u32, Vec<u32>>::with_capacity(nodes_by_id.len());
    for node in nodes_by_id.values() {
        for &child_node_id in &node.child_node_ids {
            parent_node_ids_by_child
                .entry(child_node_id)
                .or_default()
                .push(node.node_id);
        }
    }

    let mut clusters_by_subtree_node = HashMap::with_capacity(nodes_by_id.len());
    let mut pending_node_ids = Vec::new();
    let mut visited_node_ids = HashSet::new();
    for cluster in leaf_clusters {
        pending_node_ids.clear();
        visited_node_ids.clear();
        pending_node_ids.push(cluster.node_id());
        while let Some(node_id) = pending_node_ids.pop() {
            if !visited_node_ids.insert(node_id) {
                continue;
            }
            clusters_by_subtree_node
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push(cluster);
            if let Some(parent_node_ids) = parent_node_ids_by_child.get(&node_id) {
                pending_node_ids.extend(parent_node_ids.iter().copied());
            }
        }
    }
    clusters_by_subtree_node
}

#[cfg(test)]
mod performance_tests;
