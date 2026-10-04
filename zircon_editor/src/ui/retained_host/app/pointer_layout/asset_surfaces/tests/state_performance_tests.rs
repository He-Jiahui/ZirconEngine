#[test]
fn pointer_snapshot_reuses_the_committed_asset_projection() {
    let source = include_str!("../state.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(
        !production.contains("self.runtime.editor_snapshot()"),
        "pointer-frequency callbacks must not build the complete editor snapshot"
    );
}

#[test]
fn pointer_size_changes_patch_geometry_without_rebuilding_list_projections() {
    let content = include_str!("../../../asset_content_pointer/target/dispatch.rs");
    let tree = include_str!("../../../asset_tree_pointer/target.rs");
    let reference = include_str!("../../../asset_reference_pointer/target/dispatch.rs");

    for source in [content, tree, reference] {
        assert!(source.contains("sync_pane_size"));
        assert!(!source.contains("from_snapshot"));
        assert!(!source.contains("from_references"));
    }
    assert!(content.contains("if surface.content_size == target.content_size"));
    assert!(tree.contains("if surface.tree_size == tree_size"));
    assert!(reference.contains("if list.size == target.list_size"));
}
