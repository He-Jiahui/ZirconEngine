#[test]
fn geometry_builder_does_not_invoke_semantic_pane_projection() {
    let source = include_str!("../geometry.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("geometry production source");
    let function = production
        .split("pub(crate) fn build_host_scene_geometry")
        .nth(1)
        .expect("geometry builder source");

    assert!(function.contains("current.clone()"));
    assert!(function.contains("rebuild_left_dock_geometry"));
    for forbidden in [
        "pane_with_host_owned_shell_layouts(",
        "pane_with_ui_asset_nodes(",
        "pane_with_hierarchy_projection(",
        "pane_with_inspector_projection(",
        "pane_with_assets_activity_projection(",
        "pane_with_asset_browser_projection(",
        "pane_with_project_overview_projection(",
        "pane_with_animation_projection(",
        "floating_windows_with_pane_shell_layouts(",
    ] {
        assert!(
            !function.contains(forbidden),
            "geometry builder must not invoke {forbidden}"
        );
    }
}
