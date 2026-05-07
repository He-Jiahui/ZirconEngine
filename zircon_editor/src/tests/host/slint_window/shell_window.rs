use std::rc::Rc;

use crate::ui::slint_host::{
    paint_runtime_render_commands_for_test, FloatingWindowData, FrameRect,
    HostBottomDockSurfaceData, HostChromeControlFrameData, HostDocumentDockSurfaceData,
    HostFloatingWindowLayerData, HostMenuChromeData, HostMenuChromeMenuData,
    HostSideDockSurfaceData, HostStatusBarData, HostWindowLayoutData, HostWindowShellData,
    NewProjectFormData, PaneData, RecentProjectData, SceneViewportChromeData,
    TemplateNodeFrameData, TemplatePaneNodeData, UiHostContext, UiHostWindow, WelcomePaneData,
    STARTUP_REFRESH_DIAGNOSTICS_OVERLAY,
};
use slint::{ModelRc, PhysicalSize, SharedString, VecModel};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiTextAlign, UiTextRenderMode,
        UiTextWrap, UiVisualAssetRef,
    },
};

#[test]
fn workbench_shell_window_can_resize_and_toggle_maximize() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");

    let initial = ui.window().size();
    assert!(initial.width > 0);
    assert!(initial.height > 0);

    ui.window()
        .set_size(PhysicalSize::new(initial.width + 120, initial.height + 80));

    let resized = ui.window().size();
    let bootstrap = ui.get_host_window_bootstrap();
    assert_eq!(resized.width, initial.width + 120);
    assert_eq!(resized.height, initial.height + 80);
    assert_eq!(bootstrap.shell_frame.width, resized.width as f32);
    assert_eq!(bootstrap.shell_frame.height, resized.height as f32);

    assert!(!ui.window().is_maximized());
    ui.window().set_maximized(true);
    assert!(ui.window().is_maximized());
    ui.window().set_maximized(false);
    assert!(!ui.window().is_maximized());
}

#[test]
fn rust_owned_host_window_snapshot_contains_editor_chrome_pixels() {
    i_slint_backend_testing::init_no_event_loop();

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
    i_slint_backend_testing::init_no_event_loop();

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
    i_slint_backend_testing::init_no_event_loop();

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
    i_slint_backend_testing::init_no_event_loop();

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
    i_slint_backend_testing::init_no_event_loop();

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

#[test]
fn rust_owned_host_window_snapshot_renders_template_node_styles() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 248.0, 118.0),
        header_frame: host_frame(0.0, 0.0, 248.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 248.0, 85.0),
        pane: pane_with_nodes(
            "Inspector",
            vec![
                selected_template_node(
                    "SelectedPanel",
                    "Panel",
                    "Selected",
                    10.0,
                    10.0,
                    80.0,
                    24.0,
                ),
                primary_template_node("PrimaryButton", "Button", "Apply", 102.0, 10.0, 70.0, 24.0),
                disabled_template_node(
                    "DisabledPanel",
                    "Panel",
                    "Disabled",
                    10.0,
                    44.0,
                    80.0,
                    24.0,
                ),
                muted_label_node("MutedLabel", "Label", "Muted text", 102.0, 48.0, 96.0, 18.0),
            ],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("styled template node snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 84, 102),
        [54, 83, 130, 255],
        "selected template panel should use active surface color"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 176, 102),
        [76, 125, 213, 255],
        "primary button variant should use primary surface color"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 84, 136),
        [30, 33, 39, 255],
        "disabled template node should use disabled surface color"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 180, 148),
        [24, 29, 37, 255],
        "label-only template nodes should render deterministic text bars"
    );
}

#[test]
fn rust_owned_host_window_snapshot_renders_template_icon_states() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(260, 160));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(260.0, 160.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(260.0, 160.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(40.0, 58.0, 180.0, 78.0),
        header_frame: host_frame(0.0, 0.0, 180.0, 0.0),
        content_frame: host_frame(0.0, 0.0, 180.0, 78.0),
        pane: pane_with_nodes(
            "Inspector",
            vec![
                icon_state_node("HoveredIcon", 18.0, 16.0, false, true, false, false),
                icon_state_node("PressedIcon", 70.0, 16.0, false, false, true, false),
                icon_state_node("SelectedIcon", 122.0, 16.0, true, false, false, false),
                icon_state_node("DisabledIcon", 18.0, 50.0, false, false, false, true),
            ],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("icon-state template node snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 59, 75),
        [44, 53, 66, 255],
        "hovered icon controls should paint the Material hover state layer"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 111, 75),
        [47, 64, 94, 255],
        "pressed icon controls should paint a distinct pressed state"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 163, 75),
        [54, 83, 130, 255],
        "selected icon controls should paint the active selected surface"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 59, 109),
        [30, 33, 39, 255],
        "disabled icon controls should paint the disabled surface"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 178, 90),
        [123, 156, 224, 255],
        "selected icons should use the active icon tint"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 178, 90),
        pixel(snapshot.width(), snapshot.as_bytes(), 74, 124),
        "selected and disabled icon glyphs should be visually distinguishable"
    );
}

#[test]
fn rust_owned_host_window_snapshot_respects_template_node_order_and_clip() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut baseline = ui.get_host_presentation();
    baseline.host_layout = host_window_layout_for_test(320.0, 200.0);
    baseline.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    baseline.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 120.0, 118.0),
        header_frame: host_frame(0.0, 0.0, 120.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 120.0, 85.0),
        pane: pane_with_nodes("Inspector", Vec::new()),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(baseline.clone());
    let baseline_snapshot = ui
        .window()
        .take_snapshot()
        .expect("baseline order/clip snapshot should render");

    let mut with_nodes = baseline;
    with_nodes.host_scene_data.document_dock.pane = pane_with_nodes(
        "Inspector",
        vec![
            disabled_template_node("BackPanel", "Panel", "Back", 10.0, 10.0, 58.0, 28.0),
            selected_template_node("FrontPanel", "Panel", "Front", 10.0, 10.0, 58.0, 28.0),
            primary_template_node("ClippedPanel", "Panel", "Clip", 100.0, 44.0, 80.0, 24.0),
        ],
    );
    ui.set_host_presentation(with_nodes);
    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("order/clip template node snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 92, 106),
        [54, 83, 130, 255],
        "later overlapping template nodes should paint over earlier nodes"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 182, 138),
        pixel(
            baseline_snapshot.width(),
            baseline_snapshot.as_bytes(),
            182,
            138
        ),
        "node portion inside pane clip should paint"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 202, 138),
        pixel(
            baseline_snapshot.width(),
            baseline_snapshot.as_bytes(),
            202,
            138
        ),
        "node portion outside pane clip should not paint"
    );
}

#[test]
fn rust_owned_host_painter_does_not_render_structural_control_ids_as_text() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut anonymous = ui.get_host_presentation();
    anonymous.host_layout = host_window_layout_for_test(320.0, 200.0);
    anonymous.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    anonymous.host_scene_data.page_chrome.template_nodes =
        model_rc(vec![template_node("", "Panel", "", 0.0, 26.0, 320.0, 30.0)]);
    ui.set_host_presentation(anonymous.clone());
    let anonymous_snapshot = ui
        .window()
        .take_snapshot()
        .expect("anonymous structural panel snapshot should render");

    let mut with_control_id = anonymous;
    with_control_id.host_scene_data.page_chrome.template_nodes = model_rc(vec![template_node(
        "WorkbenchPageBar",
        "Panel",
        "",
        0.0,
        26.0,
        320.0,
        30.0,
    )]);
    ui.set_host_presentation(with_control_id.clone());
    let control_id_snapshot = ui
        .window()
        .take_snapshot()
        .expect("structural panel snapshot should render");

    let mut with_text = with_control_id;
    with_text.host_scene_data.page_chrome.template_nodes = model_rc(vec![template_node(
        "WorkbenchPageBar",
        "Panel",
        "Workbench",
        0.0,
        26.0,
        320.0,
        30.0,
    )]);
    ui.set_host_presentation(with_text);
    let text_snapshot = ui
        .window()
        .take_snapshot()
        .expect("labeled structural panel snapshot should render");

    assert_eq!(
        changed_pixel_count(
            anonymous_snapshot.width(),
            anonymous_snapshot.as_bytes(),
            control_id_snapshot.as_bytes(),
            0,
            26,
            160,
            30,
        ),
        0,
        "empty structural panel text should not fall back to control_id/node_id glyphs"
    );
    assert!(
        changed_pixel_count(
            text_snapshot.width(),
            anonymous_snapshot.as_bytes(),
            text_snapshot.as_bytes(),
            0,
            26,
            160,
            30,
        ) > 0,
        "explicit text should still render glyphs for labeled nodes"
    );
}

#[test]
fn rust_owned_host_painter_draws_runtime_render_commands() {
    let commands = vec![
        runtime_quad_command(1, 10.0, 10.0, 70.0, 36.0, 0, "#112233", "#89abcd"),
        runtime_quad_command(2, 20.0, 20.0, 38.0, 24.0, 4, "#44aa66", "#44aa66"),
        runtime_text_command(3, 12.0, 58.0, 130.0, 20.0, "Runtime Text"),
        runtime_image_command(4, 92.0, 12.0, 34.0, 34.0),
    ];

    let bytes = paint_runtime_render_commands_for_test(150, 90, &commands);

    assert_eq!(pixel(150, &bytes, 16, 16), [17, 34, 51, 255]);
    assert_eq!(
        pixel(150, &bytes, 28, 28),
        [68, 170, 102, 255],
        "higher z-index runtime commands should paint over lower commands"
    );
    assert!(
        lit_row_count(150, &bytes, 12, 58, 130, 20) > 0,
        "runtime text command should draw glyph pixels"
    );
    assert!(
        lit_row_count(150, &bytes, 0, 50, 145, 36) > 6,
        "runtime text should occupy glyph-height rows instead of a 3px placeholder bar"
    );
    assert_ne!(
        pixel(150, &bytes, 108, 28),
        [0, 0, 0, 255],
        "runtime image command should draw resolved icon pixels"
    );
}

#[test]
fn rust_owned_host_painter_resolves_runtime_svg_image_assets() {
    let image = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            41,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Image("ui/editor/showcase_checker.svg".to_string()),
        )],
    );
    let fallback = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            42,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Image("missing/not-found.svg".to_string()),
        )],
    );
    let res_icon_as_image = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            43,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Image("res://icons/ionicons/options-outline.svg".to_string()),
        )],
    );
    let ionicons_icon_alias = paint_runtime_render_commands_for_test(
        80,
        56,
        &[runtime_image_command_with_asset(
            44,
            10.0,
            10.0,
            36.0,
            36.0,
            UiVisualAssetRef::Icon("ionicons/options-outline.svg".to_string()),
        )],
    );

    assert_ne!(
        pixel(80, &image, 28, 28),
        pixel(80, &fallback, 28, 28),
        "runtime SVG image assets should draw decoded pixels instead of the deterministic placeholder"
    );
    assert_eq!(
        pixel(80, &image, 16, 16),
        [77, 137, 255, 255],
        "showcase checker SVG should preserve decoded RGBA image color"
    );
    assert_ne!(
        pixel(80, &res_icon_as_image, 28, 28),
        pixel(80, &fallback, 28, 28),
        "res:// icon SVG image aliases should resolve through the editor assets tree"
    );
    assert_ne!(
        pixel(80, &ionicons_icon_alias, 28, 28),
        pixel(80, &fallback, 28, 28),
        "ionicons/name.svg icon aliases should resolve through the icon asset tree"
    );
}

#[test]
fn native_host_pointer_input_forwards_menu_callbacks_and_requests_redraw() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(320, 200));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 0.0, 0.0, 80.0, 24.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData::default()]),
        ..HostMenuChromeData::default()
    };
    ui.set_host_presentation(presentation);

    let events = Rc::new(std::cell::RefCell::new(Vec::new()));
    let host = ui.global::<UiHostContext>();
    {
        let events = events.clone();
        let ui = ui.clone_strong();
        host.on_menu_pointer_moved(move |x, y| {
            events.borrow_mut().push(format!("move:{x:.0},{y:.0}"));
            let mut menu_state = ui.get_menu_state();
            menu_state.hovered_menu_index = 0;
            ui.global::<UiHostContext>().set_menu_state(menu_state);
        });
    }
    {
        let events = events.clone();
        host.on_menu_pointer_clicked(move |x, y| {
            events.borrow_mut().push(format!("click:{x:.0},{y:.0}"))
        });
    }
    {
        let events = events.clone();
        host.on_menu_pointer_scrolled(move |x, y, delta| {
            events
                .borrow_mut()
                .push(format!("scroll:{x:.0},{y:.0},{delta:.0}"));
        });
    }

    let moved = ui.dispatch_native_pointer_move_for_test(18.0, 12.0);
    let clicked = ui.dispatch_native_primary_press_for_test(18.0, 12.0);
    let scrolled = ui.dispatch_native_pointer_scroll_for_test(18.0, 12.0, 42.0);

    assert!(moved.request_redraw());
    assert!(clicked.request_redraw());
    assert!(scrolled.request_redraw());
    assert_eq!(
        events.borrow().as_slice(),
        ["move:18,12", "click:18,12", "scroll:18,12,42"]
    );
}

#[test]
fn native_host_frame_request_invokes_installed_frame_callback_before_presenting() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    let callback_hits = Rc::new(std::cell::Cell::new(0));
    let host = ui.global::<UiHostContext>();
    {
        let callback_hits = callback_hits.clone();
        let ui = ui.clone_strong();
        host.on_frame_requested(move || {
            callback_hits.set(callback_hits.get() + 1);
            let mut presentation = ui.get_host_presentation();
            presentation.host_shell.status_secondary = "Frame callback applied".into();
            ui.set_host_presentation(presentation);
        });
    }

    ui.request_host_frame_for_test();

    assert_eq!(callback_hits.get(), 1);
    assert_eq!(
        ui.get_host_presentation().host_shell.status_secondary,
        SharedString::from("Frame callback applied")
    );
}

#[test]
fn rust_owned_host_painter_renders_distinct_glyph_shapes_instead_of_text_bars() {
    let first = paint_runtime_render_commands_for_test(
        90,
        54,
        &[runtime_text_command(11, 8.0, 8.0, 74.0, 32.0, "AB")],
    );
    let second = paint_runtime_render_commands_for_test(
        90,
        54,
        &[runtime_text_command(12, 8.0, 8.0, 74.0, 32.0, "C@")],
    );

    assert_ne!(
        first, second,
        "same-length same-byte-sum strings should render as distinct glyph shapes, not seeded bars"
    );
    assert!(
        lit_row_count(90, &first, 0, 0, 90, 54) > 8,
        "glyph rendering should cover the font's vertical extent"
    );
    assert!(
        has_antialias_pixel(&first, [254, 220, 186, 255]),
        "glyph rendering should include alpha coverage pixels instead of solid rectangles only"
    );
}

fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, width, height - 82.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: host_frame(0.0, 58.0, 72.0, height - 82.0),
        document_region_frame: host_frame(72.0, 58.0, width - 72.0, height - 82.0),
        viewport_content_frame: host_frame(88.0, 90.0, width - 104.0, height - 124.0),
        ..HostWindowLayoutData::default()
    }
}

fn scene_test_layout() -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, 420.0, 178.0),
        status_bar_frame: host_frame(0.0, 236.0, 420.0, 24.0),
        left_region_frame: host_frame(0.0, 58.0, 76.0, 178.0),
        document_region_frame: host_frame(76.0, 58.0, 250.0, 178.0),
        right_region_frame: host_frame(326.0, 58.0, 94.0, 178.0),
        bottom_region_frame: host_frame(76.0, 202.0, 250.0, 34.0),
        viewport_content_frame: host_frame(76.0, 90.0, 250.0, 146.0),
        ..HostWindowLayoutData::default()
    }
}

fn scene_pane() -> PaneData {
    PaneData {
        kind: "Scene".into(),
        title: "Scene".into(),
        show_toolbar: true,
        viewport: SceneViewportChromeData {
            tool: "Move".into(),
            transform_space: "Global".into(),
            display_mode: "Lit".into(),
            grid_mode: "Grid".into(),
            ..SceneViewportChromeData::default()
        },
        ..PaneData::default()
    }
}

fn pane_with_nodes(kind: &str, nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let node_model = model_rc(nodes);
    let mut pane = PaneData {
        kind: kind.into(),
        title: kind.into(),
        ..PaneData::default()
    };
    match kind {
        "Hierarchy" => pane.hierarchy.nodes = node_model,
        "Inspector" => pane.inspector.nodes = node_model,
        "Console" => pane.console.nodes = node_model,
        "Assets" => pane.assets_activity.nodes = node_model,
        "AssetBrowser" => pane.asset_browser.nodes = node_model,
        "Project" => pane.project_overview.nodes = node_model,
        "RuntimeDiagnostics" => pane.runtime_diagnostics.nodes = node_model,
        "ModulePlugins" => pane.module_plugins.nodes = node_model,
        "BuildExport" => pane.build_export.nodes = node_model,
        "UiAssetEditor" => pane.ui_asset.nodes = node_model,
        "AnimationSequenceEditor" | "AnimationGraphEditor" => pane.animation.nodes = node_model,
        _ => {}
    }
    pane
}

fn welcome_pane_with_content() -> PaneData {
    PaneData {
        kind: "Welcome".into(),
        title: "Welcome".into(),
        welcome: WelcomePaneData {
            title: "Open or Create".into(),
            subtitle: "Recent projects and a renderable empty-project template".into(),
            status_message: "No recent project".into(),
            form: NewProjectFormData {
                project_name: "ZirconProject".into(),
                location: "C:/Users/Tester/Documents/ZirconProjects".into(),
                project_path_preview: "C:/Users/Tester/Documents/ZirconProjects/ZirconProject"
                    .into(),
                template_label: "Renderable Empty".into(),
                validation_message: "Project settings are valid".into(),
                can_create: true,
                can_open_existing: true,
                browse_supported: true,
            },
            recent_projects: model_rc(vec![RecentProjectData {
                display_name: "ZirconProject4".into(),
                path: "C:/Users/Tester/Documents/ZirconProjects/ZirconProject4".into(),
                last_opened_label: "Reopened".into(),
                status_label: "".into(),
                invalid: false,
            }]),
            nodes: model_rc(vec![
                template_node("WelcomeOuterPanel", "Panel", "", 16.0, 12.0, 516.0, 220.0),
                template_node("WelcomeRecentPanel", "Panel", "", 16.0, 12.0, 180.0, 220.0),
                template_node(
                    "WelcomeRecentHeaderPanel",
                    "Panel",
                    "",
                    16.0,
                    24.0,
                    180.0,
                    54.0,
                ),
                template_node(
                    "WelcomeRecentListPanel",
                    "Panel",
                    "",
                    26.0,
                    92.0,
                    160.0,
                    130.0,
                ),
                template_node("WelcomeMainPanel", "Panel", "", 196.0, 12.0, 336.0, 220.0),
                template_node("WelcomeHeroPanel", "Panel", "", 224.0, 24.0, 280.0, 54.0),
                template_node("WelcomeStatusPanel", "Panel", "", 224.0, 84.0, 280.0, 30.0),
                template_node(
                    "WelcomeNewProjectHeaderPanel",
                    "Panel",
                    "",
                    224.0,
                    124.0,
                    280.0,
                    34.0,
                ),
                template_node(
                    "WelcomeProjectNameField",
                    "Panel",
                    "",
                    224.0,
                    162.0,
                    280.0,
                    44.0,
                ),
                template_node(
                    "WelcomeLocationField",
                    "Panel",
                    "",
                    224.0,
                    212.0,
                    280.0,
                    44.0,
                ),
                template_node(
                    "WelcomePreviewPanel",
                    "Panel",
                    "",
                    224.0,
                    262.0,
                    280.0,
                    50.0,
                ),
                template_node(
                    "WelcomeValidationPanel",
                    "Panel",
                    "",
                    224.0,
                    318.0,
                    280.0,
                    24.0,
                ),
                template_node("WelcomeActionsRow", "Panel", "", 224.0, 346.0, 280.0, 32.0),
            ]),
        },
        ..PaneData::default()
    }
}

fn selected_template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        selected: true,
        focused: true,
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

fn primary_template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        button_variant: "primary".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

fn disabled_template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        disabled: true,
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

fn icon_state_node(
    control_id: &str,
    x: f32,
    y: f32,
    selected: bool,
    hovered: bool,
    pressed: bool,
    disabled: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "IconButton".into(),
        icon_name: "options-outline".into(),
        has_preview_image: true,
        preview_image: solid_test_icon_image(),
        selected,
        hovered,
        pressed,
        disabled,
        border_width: 1.0,
        corner_radius: 5.0,
        ..template_node(control_id, "IconButton", "", x, y, 32.0, 32.0)
    }
}

fn solid_test_icon_image() -> slint::Image {
    let pixels = [[255, 255, 255, 255]; 4].concat();
    slint::Image::from_rgba8(
        slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&pixels, 2, 2),
    )
}

fn muted_label_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        surface_variant: "".into(),
        border_width: 0.0,
        text_tone: "muted".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

fn template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

fn control_frame(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeControlFrameData {
    HostChromeControlFrameData {
        control_id: control_id.into(),
        frame: host_frame(x, y, width, height),
    }
}

fn runtime_quad_command(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    z_index: i32,
    background: &str,
    border: &str,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(x, y, width, height),
        clip_frame: None,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.to_string()),
            border_color: Some(border.to_string()),
            border_width: 1.0,
            ..runtime_style()
        },
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

fn runtime_text_command(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: &str,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(x, y, width, height),
        clip_frame: Some(UiFrame::new(x, y, width, height)),
        z_index: 8,
        style: UiResolvedStyle {
            foreground_color: Some("#fedcba".to_string()),
            font_size: 12.0,
            line_height: 14.0,
            text_align: UiTextAlign::Left,
            wrap: UiTextWrap::None,
            text_render_mode: UiTextRenderMode::Auto,
            ..runtime_style()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}

fn lit_row_count(
    width: u32,
    bytes: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((bytes.len() / 4 / width as usize) as u32);
    (y..y1)
        .filter(|row| (x..x1).any(|column| pixel(width, bytes, column, *row) != [0, 0, 0, 255]))
        .count()
}

fn changed_pixel_count(
    width: u32,
    left: &[u8],
    right: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((left.len() / 4 / width as usize) as u32)
        .min((right.len() / 4 / width as usize) as u32);
    (y..y1)
        .flat_map(|row| (x..x1).map(move |column| (column, row)))
        .filter(|(column, row)| {
            pixel(width, left, *column, *row) != pixel(width, right, *column, *row)
        })
        .count()
}

fn has_antialias_pixel(bytes: &[u8], foreground: [u8; 4]) -> bool {
    bytes.chunks_exact(4).any(|pixel| {
        let pixel = [pixel[0], pixel[1], pixel[2], pixel[3]];
        pixel != [0, 0, 0, 255] && pixel != foreground
    })
}

fn runtime_image_command(node_id: u64, x: f32, y: f32, width: f32, height: f32) -> UiRenderCommand {
    runtime_image_command_with_asset(
        node_id,
        x,
        y,
        width,
        height,
        UiVisualAssetRef::Icon("options-outline".to_string()),
    )
}

fn runtime_image_command_with_asset(
    node_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    image: UiVisualAssetRef,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(x, y, width, height),
        clip_frame: None,
        z_index: 2,
        style: runtime_style(),
        text_layout: None,
        text: None,
        image: Some(image),
        opacity: 1.0,
    }
}

fn runtime_style() -> UiResolvedStyle {
    UiResolvedStyle::default()
}

fn pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
