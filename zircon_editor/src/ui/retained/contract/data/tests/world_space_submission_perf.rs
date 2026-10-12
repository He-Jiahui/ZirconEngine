#[test]
fn world_space_collection_borrows_host_rows_and_surface_ids() {
    let node = include_str!("../world/builder/node.rs");
    let pane = include_str!("../world/builder/pane.rs");
    let scene = include_str!("../world/builder/scene.rs");
    let compact_node = node.split_whitespace().collect::<String>();

    assert!(compact_node.contains("letmutsubmissions=nodes.iter()"));
    assert!(!node.contains("row_data("));
    assert!(!pane.contains("pane_surface_id.clone()"));
    assert!(scene.contains("floating_windows.iter()"));
    assert!(!scene.contains("floating_windows.row_data("));
}
