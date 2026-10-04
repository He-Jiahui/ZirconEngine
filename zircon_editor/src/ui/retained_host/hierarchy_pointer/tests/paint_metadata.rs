use super::*;

#[test]
fn viewport_candidates_preserve_source_order_and_ignore_unrelated_nodes() {
    let metadata = hierarchy_paint_metadata(
        [
            "HierarchyHeaderPanel",
            "HierarchyTreeSlotAnchor",
            "HierarchyListPanel",
            "SelectRoot",
        ]
        .into_iter(),
    );

    assert_eq!(metadata.viewport_node_rows(), &[1, 2]);
}

#[test]
fn duplicate_anchor_identities_publish_only_the_first_source_row() {
    let metadata = hierarchy_paint_metadata(
        [
            "HierarchyListPanel",
            "HierarchyListPanel",
            "HierarchyTreeSlotAnchor",
            "HierarchyTreeSlotAnchor",
        ]
        .into_iter(),
    );

    assert_eq!(metadata.viewport_node_rows(), &[0, 2]);
}
