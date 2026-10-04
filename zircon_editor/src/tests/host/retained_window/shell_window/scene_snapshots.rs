use super::support::*;

#[test]
fn rust_owned_host_window_snapshot_contains_editor_chrome_pixels() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout.center_band_frame = host_frame(0.0, 38.0, 320.0, 138.0);
    presentation.host_layout.left_region_frame = host_frame(0.0, 38.0, 72.0, 138.0);
    presentation.host_layout.document_region_frame = host_frame(72.0, 38.0, 248.0, 138.0);
    presentation.host_layout.status_bar_frame = host_frame(0.0, 176.0, 320.0, 24.0);
    presentation.host_layout.viewport_content_frame = host_frame(88.0, 58.0, 216.0, 96.0);
    presentation.host_shell.project_path = "res://sandbox".into();
    presentation.host_shell.status_secondary = "Ready".into();
    presentation.host_shell.viewport_label = "Scene".into();
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("rust-owned host painter should capture editor chrome");

    assert_eq!(snapshot.width(), 320);
    assert_eq!(snapshot.height(), 200);
    assert_eq!(snapshot.as_bytes().len(), 320 * 200 * 4);
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 8, 8),
        [255, 255, 255, 255]
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 8, 8),
        pixel(snapshot.width(), snapshot.as_bytes(), 96, 64)
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 96, 64),
        pixel(snapshot.width(), snapshot.as_bytes(), 12, 188)
    );
    assert!(
        snapshot
            .as_bytes()
            .chunks_exact(4)
            .any(|pixel| pixel[3] == 255),
        "snapshot should contain opaque painted pixels instead of an empty surface"
    );
}

#[test]
fn rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 120));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(320.0, 120.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(320.0, 120.0);
    presentation.host_shell = HostWindowShellData {
        project_path: "res://debug".into(),
        status_secondary: "Ready".into(),
        debug_refresh_rate: STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.into(),
        viewport_label: "Scene".into(),
        ..HostWindowShellData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("debug refresh-rate overlay should render");

    assert!(
        lit_row_count(snapshot.width(), snapshot.as_bytes(), 196, 4, 118, 28) > 4,
        "top-right debug refresh-rate marker should draw visible pixels"
    );
}

#[test]
fn rust_owned_host_window_snapshot_consumes_host_scene_data() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(420, 260));

    let mut baseline = ui.get_host_presentation();
    baseline.host_layout = scene_test_layout();
    baseline.host_shell.project_path = "res://scene-test".into();
    baseline.host_shell.status_secondary = "Ready".into();
    baseline.host_shell.viewport_label = "Scene".into();
    ui.set_host_presentation(baseline.clone());
    let baseline_snapshot = ui
        .window()
        .take_snapshot()
        .expect("baseline host painter should capture editor chrome");

    let mut presentation = baseline;
    presentation.host_scene_data.layout = scene_test_layout();
    presentation.host_scene_data.menu_chrome.template_nodes = model_rc(vec![template_node(
        "WorkbenchMenuTopBar",
        "Panel",
        "File Edit Selection Play",
        0.0,
        0.0,
        420.0,
        25.0,
    )]);
    presentation.host_scene_data.page_chrome.template_nodes =
        model_rc(vec![selected_template_node(
            "PageTab0",
            "Button",
            "Workbench",
            8.0,
            27.0,
            108.0,
            30.0,
        )]);
    presentation.host_scene_data.status_bar = HostStatusBarData {
        status_bar_frame: host_frame(0.0, 236.0, 420.0, 24.0),
        template_nodes: model_rc(vec![template_node(
            "StatusPrimaryLabel",
            "Label",
            "Scene data active",
            12.0,
            4.0,
            160.0,
            14.0,
        )]),
        status_primary: "Scene data active".into(),
        status_secondary: "Ready".into(),
        viewport_label: "Scene".into(),
    };
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData {
        region_frame: host_frame(0.0, 58.0, 76.0, 178.0),
        rail_before_panel: true,
        rail_width_px: 34.0,
        panel_width_px: 42.0,
        panel_header_height_px: 31.0,
        rail_nodes: model_rc(vec![selected_template_node(
            "ActivityRailButton0",
            "Button",
            "PR",
            3.0,
            8.0,
            28.0,
            28.0,
        )]),
        header_frame: host_frame(0.0, 0.0, 42.0, 31.0),
        header_nodes: model_rc(vec![template_node(
            "DockTab0", "Button", "Project", 2.0, 1.0, 38.0, 29.0,
        )]),
        content_frame: host_frame(0.0, 32.0, 42.0, 145.0),
        pane: pane_with_nodes(
            "Hierarchy",
            vec![template_node(
                "HierarchyRow",
                "Panel",
                "Camera",
                2.0,
                5.0,
                38.0,
                18.0,
            )],
        ),
        ..HostSideDockSurfaceData::default()
    };
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(76.0, 58.0, 250.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 250.0, 31.0),
        header_nodes: model_rc(vec![selected_template_node(
            "DockTab0", "Button", "Scene", 8.0, 1.0, 92.0, 30.0,
        )]),
        content_frame: host_frame(0.0, 32.0, 250.0, 145.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    presentation.host_scene_data.bottom_dock = HostBottomDockSurfaceData {
        region_frame: host_frame(76.0, 202.0, 250.0, 34.0),
        header_frame: host_frame(0.0, 0.0, 250.0, 31.0),
        header_nodes: model_rc(vec![template_node(
            "DockTab0", "Button", "Console", 8.0, 1.0, 92.0, 30.0,
        )]),
        content_frame: host_frame(0.0, 32.0, 250.0, 1.0),
        pane: pane_with_nodes("Console", Vec::new()),
        expanded: true,
        header_height_px: 31.0,
        ..HostBottomDockSurfaceData::default()
    };
    presentation.host_scene_data.floating_layer = HostFloatingWindowLayerData {
        floating_windows: model_rc(vec![FloatingWindowData {
            window_id: "floating.inspector".into(),
            title: "Inspector".into(),
            frame: host_frame(164.0, 84.0, 128.0, 92.0),
            header_frame: host_frame(0.0, 0.0, 128.0, 31.0),
            header_nodes: model_rc(vec![selected_template_node(
                "DockTab0",
                "Button",
                "Inspector",
                8.0,
                1.0,
                92.0,
                30.0,
            )]),
            active_pane: pane_with_nodes(
                "Inspector",
                vec![template_node(
                    "InspectorField",
                    "Panel",
                    "Transform",
                    4.0,
                    6.0,
                    96.0,
                    18.0,
                )],
            ),
            ..FloatingWindowData::default()
        }]),
        header_height_px: 31.0,
    };
    ui.set_host_presentation(presentation);
    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("data-driven host painter should capture scene DTOs");

    for (x, y, label) in [
        (12, 10, "menu chrome"),
        (20, 42, "page tab"),
        (8, 70, "activity rail"),
        (44, 66, "dock header"),
        (96, 106, "viewport toolbar"),
        (16, 244, "status bar"),
        (174, 94, "floating header"),
    ] {
        assert_ne!(
            pixel(snapshot.width(), snapshot.as_bytes(), x, y),
            pixel(
                baseline_snapshot.width(),
                baseline_snapshot.as_bytes(),
                x,
                y
            ),
            "{label} should be painted from host scene data rather than the skeletal fallback"
        );
    }
}

#[test]
fn rust_owned_host_window_snapshot_reflects_pane_template_nodes() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut baseline = ui.get_host_presentation();
    baseline.host_layout = host_window_layout_for_test(320.0, 200.0);
    baseline.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    baseline.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 248.0, 118.0),
        header_frame: host_frame(0.0, 0.0, 248.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 248.0, 85.0),
        pane: pane_with_nodes("Inspector", Vec::new()),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(baseline.clone());
    let baseline_snapshot = ui
        .window()
        .take_snapshot()
        .expect("baseline pane snapshot should render");

    let mut with_nodes = baseline;
    with_nodes.host_scene_data.document_dock.pane = pane_with_nodes(
        "Inspector",
        vec![selected_template_node(
            "InspectorTransformRow",
            "Panel",
            "Transform Position",
            10.0,
            10.0,
            180.0,
            24.0,
        )],
    );
    ui.set_host_presentation(with_nodes);
    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("pane template node snapshot should render");

    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 92, 104),
        pixel(
            baseline_snapshot.width(),
            baseline_snapshot.as_bytes(),
            92,
            104
        ),
        "pane body template nodes should change native host pixels"
    );
}

#[test]
fn rust_owned_host_window_snapshot_draws_welcome_main_content() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(640, 360));

    let mut baseline = ui.get_host_presentation();
    baseline.host_layout = host_window_layout_for_test(640.0, 360.0);
    baseline.host_scene_data.layout = host_window_layout_for_test(640.0, 360.0);
    baseline.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 548.0, 278.0),
        header_frame: host_frame(0.0, 0.0, 548.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 548.0, 245.0),
        pane: PaneData {
            kind: "Welcome".into(),
            title: "Welcome".into(),
            ..PaneData::default()
        },
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(baseline.clone());
    let fallback_snapshot = ui
        .window()
        .take_snapshot()
        .expect("fallback welcome snapshot should render");

    let mut with_welcome = baseline;
    with_welcome.host_scene_data.document_dock.pane = welcome_pane_with_content();
    ui.set_host_presentation(with_welcome);
    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("welcome content snapshot should render");

    assert!(
        changed_pixel_count(
            snapshot.width(),
            fallback_snapshot.as_bytes(),
            snapshot.as_bytes(),
            88,
            102,
            510,
            218,
        ) > 2400,
        "welcome pane should render the projected Material/Slate content instead of the fallback label"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 318, 266),
        pixel(
            fallback_snapshot.width(),
            fallback_snapshot.as_bytes(),
            318,
            266
        ),
        "new-project field area should contain visible native host paint"
    );
}
