#[cfg(test)]
#[test]
fn stable_assets_activity_snapshot_reuses_the_composed_model() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let size = UiSize::new(420.0, 360.0);

    let first = assets_activity_pane_data(&snapshot, size);
    let stable = assets_activity_pane_data(&snapshot, size);

    assert!(first.nodes.shares_values_with(&stable.nodes));
}
