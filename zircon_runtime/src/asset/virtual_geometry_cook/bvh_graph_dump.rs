use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::asset::{
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryHierarchyNodeAsset,
};

pub fn format_virtual_geometry_cook_bvh_graph_dump(asset: &VirtualGeometryAsset) -> String {
    let mut graph = String::new();
    let cluster_ids_by_node = cluster_ids_by_node(asset);

    write_line(&mut graph, format_args!("digraph virtual_geometry_bvh {{"));
    write_line(&mut graph, format_args!("  graph [rankdir=TB];"));
    write_line(&mut graph, format_args!("  node [fontname=\"monospace\"];"));
    for node in sorted_hierarchy_nodes(asset) {
        write_node(&mut graph, node, &cluster_ids_by_node);
    }
    for node in sorted_hierarchy_nodes(asset) {
        for child_node_id in &node.child_node_ids {
            write_line(
                &mut graph,
                format_args!("  node_{} -> node_{};", node.node_id, child_node_id),
            );
        }
    }
    write_line(&mut graph, format_args!("}}"));

    graph
}

fn write_node(
    graph: &mut String,
    node: &VirtualGeometryHierarchyNodeAsset,
    cluster_ids_by_node: &BTreeMap<u32, Vec<u32>>,
) {
    let cluster_ids = cluster_ids_by_node
        .get(&node.node_id)
        .cloned()
        .unwrap_or_default();
    let shape = if node.child_node_ids.is_empty() {
        "ellipse"
    } else {
        "box"
    };
    write_line(
        graph,
        format_args!(
            "  node_{} [shape={} label=\"node {}\\nmip {}\\npage {}\\nclusters {}\\nsse {:.6}\"];",
            node.node_id,
            shape,
            node.node_id,
            node.mip_level,
            node.page_id,
            format_u32_list(&cluster_ids),
            node.screen_space_error
        ),
    );
}

fn sorted_hierarchy_nodes(asset: &VirtualGeometryAsset) -> Vec<&VirtualGeometryHierarchyNodeAsset> {
    let mut nodes = asset.hierarchy_buffer.iter().collect::<Vec<_>>();
    nodes.sort_by_key(|node| node.node_id);
    nodes
}

fn cluster_ids_by_node(asset: &VirtualGeometryAsset) -> BTreeMap<u32, Vec<u32>> {
    let mut clusters = asset.cluster_headers.iter().collect::<Vec<_>>();
    clusters.sort_by_key(|cluster| cluster.cluster_id);
    let mut cluster_ids_by_node = BTreeMap::<u32, Vec<u32>>::new();
    for cluster in clusters {
        push_cluster_id(&mut cluster_ids_by_node, cluster);
    }
    cluster_ids_by_node
}

fn push_cluster_id(
    cluster_ids_by_node: &mut BTreeMap<u32, Vec<u32>>,
    cluster: &VirtualGeometryClusterHeaderAsset,
) {
    cluster_ids_by_node
        .entry(cluster.hierarchy_node_id)
        .or_default()
        .push(cluster.cluster_id);
}

fn format_u32_list(values: &[u32]) -> String {
    let mut formatted = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            formatted.push(',');
        }
        write!(formatted, "{value}").expect("writing to String cannot fail");
    }
    formatted.push(']');
    formatted
}

fn write_line(graph: &mut String, args: std::fmt::Arguments<'_>) {
    writeln!(graph, "{args}").expect("writing to String cannot fail");
}
