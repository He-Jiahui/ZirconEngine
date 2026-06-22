use super::*;

#[test]
fn apply_presentation_ignores_root_projection_frames_when_workbench_layout_frames_are_missing() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = WorkbenchShellGeometry::default();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.center_band_frame,
        crate::ui::retained_host::FrameRect::default()
    );
    assert_eq!(
        host_layout.document_region_frame,
        crate::ui::retained_host::FrameRect::default()
    );
    assert_eq!(
        host_layout.status_bar_frame,
        crate::ui::retained_host::FrameRect::default()
    );
    assert_eq!(
        host_layout.viewport_content_frame,
        crate::ui::retained_host::FrameRect::default()
    );
}

#[test]
fn apply_presentation_projects_default_design_stack_ids_into_host_shell() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let geometry = WorkbenchShellGeometry::default();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &["Default".to_string()],
        Some("Default"),
        &ui_asset_panes,
        &animation_panes,
        None,
        None,
        &floating_window_projection_bundle,
    );

    let host_shell = ui.get_host_presentation().host_shell;
    assert_eq!(host_shell.active_preset_name, "Default");
    assert_eq!(host_shell.skin_id, "material_dark");
    assert_eq!(host_shell.panel_preset_id, "fyrox_panel");
    assert_eq!(host_shell.shell_preset_id, "jetbrains_shell");
    assert_eq!(host_shell.window_model_preset_id, "unreal_window_model");
    assert!(ui.get_host_presentation().menu_state.open_menu_index >= -1);
}
