use super::{
    build_source_outline_index_for_node_ids, UiAssetSourceOutlineEntry, UiAssetSourceOutlineIndex,
};

#[test]
fn outline_index_preserves_direct_block_ranges_and_line_queries() {
    let source =
        "[nodes.root]\nkind = \"container\"\nname = \"Root\"\n[nodes.label]\nkind = \"label\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

    assert_eq!(index.entries().len(), 2);
    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.line),
        Some(1)
    );
    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.end_line),
        Some(3)
    );
    assert_eq!(index.node_id_for_line(2), Some("root"));
    assert_eq!(index.node_id_for_line(4), Some("label"));
}

#[test]
fn outline_index_preserves_tree_ranges_and_node_line_queries() {
    let source = "[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n[nodes.node.style]\ncolor = \"white\"\n[[nodes]]\n[nodes.node]\nnode_id = \"label\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.line),
        Some(3)
    );
    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.end_line),
        Some(5)
    );
    assert_eq!(index.node_id_for_line(4), Some("root"));
    assert_eq!(index.node_id_for_line(8), Some("label"));
}

#[test]
fn outline_index_preserves_direct_blocks_over_tree_fallbacks() {
    let source =
        "[nodes.root]\nkind = \"container\"\n[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root"]);

    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.line),
        Some(1)
    );
    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.end_line),
        Some(2)
    );
    assert_eq!(index.node_id_for_line(4), None);
}

#[test]
fn outline_index_stops_direct_ranges_at_malformed_header_boundaries() {
    let source = "[nodes.root]\nkind = \"container\"\n[broken\nname = \"not root\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root"]);

    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.end_line),
        Some(2)
    );
    assert_eq!(index.node_id_for_line(3), None);
    assert_eq!(index.node_id_for_line(4), None);
}

#[test]
fn outline_index_freezes_tree_wrapper_before_the_node_id_line() {
    let source = "[[nodes]]\n[nodes.node]\n[[nodes]]\nnode_id = \"root\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root"]);

    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.end_line),
        Some(2)
    );
    assert_eq!(index.node_id_for_line(4), None);
}

#[test]
fn outline_index_only_matches_complete_node_path_segments_for_tree_wrappers() {
    let source = "[[nodes]]\n[nodes.nodelet]\nnode_id = \"root\"\n[[nodes]]\n";
    let index = build_source_outline_index_for_node_ids(source, ["root"]);

    assert_eq!(
        index
            .entry_for_node("root")
            .map(|entry| entry.excerpt.as_str()),
        Some("[nodes.root]\n[nodes.nodelet]\nnode_id = \"root\"")
    );
}

#[test]
fn outline_index_uses_the_complete_parent_path_for_tree_wrappers() {
    let source = "[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n[[nodes]]\n";
    let index = build_source_outline_index_for_node_ids(source, ["root"]);

    assert_eq!(
        index
            .entry_for_node("root")
            .map(|entry| entry.excerpt.as_str()),
        Some("[[nodes]]\n[nodes.node]\nnode_id = \"root\"")
    );
}

#[test]
fn outline_index_preserves_the_first_unmapped_tree_node_occurrence() {
    let source = "node_id = \"root\"\n[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root"]);

    assert_eq!(index.entry_for_node("root"), None);
}

#[test]
fn outline_index_prefers_the_most_specific_precompiled_line_segment() {
    let index = UiAssetSourceOutlineIndex::from_entries(vec![
        UiAssetSourceOutlineEntry {
            node_id: "root".to_string(),
            block_label: "[nodes.root]".to_string(),
            line: 1,
            end_line: 10,
            excerpt: String::new(),
        },
        UiAssetSourceOutlineEntry {
            node_id: "child".to_string(),
            block_label: "[nodes.child]".to_string(),
            line: 4,
            end_line: 6,
            excerpt: String::new(),
        },
        UiAssetSourceOutlineEntry {
            node_id: "same_start".to_string(),
            block_label: "[nodes.same_start]".to_string(),
            line: 4,
            end_line: 5,
            excerpt: String::new(),
        },
    ]);

    assert_eq!(index.node_id_for_line(3), Some("root"));
    assert_eq!(index.node_id_for_line(4), Some("same_start"));
    assert_eq!(index.node_id_for_line(6), Some("child"));
    assert_eq!(index.node_id_for_line(7), Some("root"));
}

#[test]
fn outline_index_skips_empty_tree_ranges_instead_of_clamping_them_to_a_node_line() {
    let index = UiAssetSourceOutlineIndex::from_entries(vec![UiAssetSourceOutlineEntry {
        node_id: "root".to_string(),
        block_label: "[nodes.root]".to_string(),
        line: 4,
        end_line: 3,
        excerpt: String::new(),
    }]);

    assert_eq!(index.node_id_for_line(3), None);
    assert_eq!(index.node_id_for_line(4), None);
}

#[test]
fn outline_index_exposes_the_sorted_entry_position_for_a_node() {
    let source = "[nodes.root]\nkind = \"container\"\n[nodes.label]\nkind = \"label\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

    assert_eq!(index.index_for_node("root"), Some(0));
    assert_eq!(index.index_for_node("label"), Some(1));
    assert_eq!(index.index_for_node("missing"), None);
}

#[test]
fn header_ranges_close_repeated_array_headers_without_rescanning_active_ranges() {
    let source = "[[nodes]]\n[nodes.node]\nnode_id = \"root\"\n[[nodes]]\n[nodes.node]\nnode_id = \"label\"\n";
    let index = build_source_outline_index_for_node_ids(source, ["root", "label"]);

    assert_eq!(
        index.entry_for_node("root").map(|entry| entry.end_line),
        Some(3)
    );
    assert_eq!(
        index.entry_for_node("label").map(|entry| entry.end_line),
        Some(6)
    );
}
