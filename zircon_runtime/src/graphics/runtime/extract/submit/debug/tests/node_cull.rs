use super::{index_hierarchy_nodes, RenderVirtualGeometryHierarchyNode};

#[test]
fn runtime94_hierarchy_node_index_preserves_first_authored_duplicate() {
    let hierarchy_nodes = [
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 11,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 1,
            node_id: 7,
            child_base: 2,
            child_count: 3,
            cluster_start: 22,
            cluster_count: 4,
        },
    ];

    let nodes_by_id = index_hierarchy_nodes(&hierarchy_nodes);

    assert_eq!(nodes_by_id.len(), 1);
    assert_eq!(nodes_by_id[&7].cluster_start, 11);
}
