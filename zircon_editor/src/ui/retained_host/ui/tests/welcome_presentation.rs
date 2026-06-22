use super::*;

#[test]
fn apply_presentation_projects_welcome_mount_nodes_into_global_context() {
    let (chrome, model, ui_asset_panes, animation_panes) = welcome_shell_fixture();
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
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            9.0, 19.0, 333.0, 444.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            15.0, 520.0, 640.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
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

    let pane_surface_frame = projection_frames
        .pane_surface_frame
        .expect("shared host projection should expose the welcome pane surface frame");
    let expected_nodes = crate::ui::layouts::views::welcome_pane_nodes(UiSize::new(
        pane_surface_frame.width.max(0.0),
        pane_surface_frame.height.max(0.0),
    ));
    let projected = ui
        .global::<crate::ui::retained_host::PaneSurfaceHostContext>()
        .get_welcome_pane();
    let expected_nodes = (0..expected_nodes.row_count())
        .filter_map(|row| expected_nodes.row_data(row))
        .collect::<Vec<_>>();
    let projected_nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert_eq!(projected.title, "Open or Create");
    assert_eq!(projected_nodes.len(), expected_nodes.len());

    for control_id in [
        "WelcomeOuterPanel",
        "WelcomeRecentPanel",
        "WelcomeMainPanel",
        "WelcomePreviewPanel",
        "WelcomeActionsRow",
    ] {
        let expected = expected_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .expect("expected welcome node");
        let actual = projected_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .expect("projected welcome node");
        assert_eq!(actual.role.to_string(), expected.role.to_string());
        assert_eq!(actual.frame.x, expected.frame.x);
        assert_eq!(actual.frame.y, expected.frame.y);
        assert_eq!(actual.frame.width, expected.frame.width);
        assert_eq!(actual.frame.height, expected.frame.height);
    }
}
