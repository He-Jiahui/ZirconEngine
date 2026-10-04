#[test]
fn visible_welcome_size_borrows_floating_window_rows() {
    let source = include_str!("../apply_presentation.rs");
    let function = source
        .split("fn resolve_visible_welcome_pane_size")
        .nth(1)
        .and_then(|body| body.split("fn dock_content_height").next())
        .expect("welcome size implementation");

    assert!(function.contains(".floating_windows"));
    assert!(function.contains(".iter()"));
    assert!(!function.contains("row_data"));
}

#[test]
fn apply_presentation_does_not_build_discarded_pane_globals() {
    let source = include_str!("../apply_presentation.rs");
    let function = source
        .split("pub(crate) fn apply_presentation")
        .nth(1)
        .and_then(|body| body.split("fn host_window_layout").next())
        .expect("apply presentation implementation");

    assert!(!function.contains("apply_pane_surface_globals"));
    assert!(!function.contains("set_activity_asset_"));
    assert!(!function.contains("set_browser_asset_"));
    assert!(!function.contains("set_recent_projects"));
    assert!(!function.contains("set_project_overview"));
}

#[test]
fn window_metrics_geometry_path_keeps_pane_projection_out_of_the_hot_path() {
    let source = include_str!("../apply_presentation.rs");
    let function = source
        .split("pub(crate) fn apply_window_metrics_geometry_presentation")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn host_window_layout").next())
        .expect("window metrics geometry implementation");

    assert!(function.contains("build_host_scene_geometry"));
    assert!(function.contains("to_host_contract_host_scene_geometry_with_retained_panes"));
    assert!(function.contains("cached.host_surface_data"));
    assert!(function.contains("cached.retained_scene_data"));
    assert!(function.contains("floating_window_projection_bundle"));
    assert!(!function.contains("build_host_scene_data_with_cache"));
    assert!(!function.contains("to_host_contract_host_scene_data_with_runtime"));
    assert!(!function.contains("PaneProjectionBuildCount"));
}
