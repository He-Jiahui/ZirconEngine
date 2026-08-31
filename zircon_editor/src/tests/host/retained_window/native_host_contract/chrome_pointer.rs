use super::support::*;

#[test]
fn native_host_pointer_click_routes_document_tab_with_document_region_origin() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        tab_frames: model_rc(vec![chrome_tab(
            "document.scene",
            "Scene",
            12.0,
            4.0,
            84.0,
            24.0,
        )]),
        tabs: model_rc(vec![tab_data("document.scene", "Scene")]),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_document_tab_pointer_clicked(move |surface_key, index, tab_x, tab_width, x, y| {
                clicks
                    .borrow_mut()
                    .push((surface_key.to_string(), index, tab_x, tab_width, x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(60.0 + 24.0, 58.0 + 12.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(60.0, 58.0, 280.0, 138.0)),
        "document tab press should refresh presentation without repainting the full native window"
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [("document".to_string(), 0, 12.0, 84.0, 24.0, 12.0)],
        "root document tabs should be hit-tested in document-region global coordinates"
    );
}

#[test]
fn native_host_close_prompt_button_press_uses_overlay_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.close_prompt = HostClosePromptData {
        visible: true,
        can_save: true,
        overlay_frame: host_frame(30.0, 40.0, 260.0, 150.0),
        dialog_frame: host_frame(70.0, 70.0, 180.0, 90.0),
        discard_button_frame: host_frame(120.0, 130.0, 60.0, 22.0),
        ..HostClosePromptData::default()
    };
    ui.set_host_presentation(presentation);

    let actions = Rc::new(RefCell::new(Vec::new()));
    {
        let actions = actions.clone();
        ui.global::<UiHostContext>()
            .on_close_prompt_action_clicked(move |action_id| {
                actions.borrow_mut().push(action_id.to_string());
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(140.0, 140.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(30.0, 40.0, 260.0, 150.0)),
        "close prompt action should repaint overlay damage without requesting an unbounded full frame"
    );
    assert_eq!(actions.borrow().as_slice(), ["discard"]);
}

#[test]
fn native_host_floating_document_tab_press_uses_floating_window_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.floating_layer.floating_windows =
        model_rc(vec![FloatingWindowData {
            window_id: "floating-a".into(),
            frame: host_frame(120.0, 70.0, 220.0, 150.0),
            header_frame: host_frame(0.0, 0.0, 220.0, 31.0),
            tab_frames: model_rc(vec![chrome_tab(
                "floating.scene",
                "Scene",
                12.0,
                4.0,
                84.0,
                24.0,
            )]),
            tabs: model_rc(vec![tab_data("floating.scene", "Scene")]),
            active_pane: scene_pane(),
            ..FloatingWindowData::default()
        }]);
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_document_tab_pointer_clicked(move |surface_key, index, tab_x, tab_width, x, y| {
                clicks
                    .borrow_mut()
                    .push((surface_key.to_string(), index, tab_x, tab_width, x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(120.0 + 24.0, 70.0 + 12.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(120.0, 70.0, 220.0, 150.0)),
        "floating document-tab press should refresh presentation while repainting only the owning floating window"
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [("floating-a".to_string(), 0, 12.0, 84.0, 24.0, 12.0)]
    );
}

#[test]
fn native_host_floating_window_header_press_uses_floating_layer_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.floating_layer.floating_windows = model_rc(vec![
        FloatingWindowData {
            window_id: "floating-a".into(),
            frame: host_frame(40.0, 70.0, 100.0, 100.0),
            header_frame: host_frame(0.0, 0.0, 100.0, 31.0),
            ..FloatingWindowData::default()
        },
        FloatingWindowData {
            window_id: "floating-b".into(),
            frame: host_frame(180.0, 90.0, 160.0, 120.0),
            header_frame: host_frame(0.0, 0.0, 160.0, 31.0),
            tab_frames: model_rc(vec![chrome_tab(
                "floating.scene",
                "Scene",
                12.0,
                4.0,
                84.0,
                24.0,
            )]),
            tabs: model_rc(vec![tab_data("floating.scene", "Scene")]),
            active_pane: scene_pane(),
            ..FloatingWindowData::default()
        },
    ]);
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_floating_window_header_pointer_clicked(move |x, y| {
                clicks.borrow_mut().push((x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(180.0 + 132.0, 90.0 + 12.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(40.0, 70.0, 300.0, 140.0)),
        "floating header focus may reorder floating windows, so damage should cover the floating layer union without repainting the full host"
    );
    assert_eq!(clicks.borrow().as_slice(), [(312.0, 102.0)]);
}

#[test]
fn native_host_drawer_header_tab_press_uses_drawer_region_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData {
        surface_key: "left".into(),
        region_frame: host_frame(0.0, 58.0, 180.0, 178.0),
        header_frame: host_frame(32.0, 0.0, 148.0, 31.0),
        tab_frames: model_rc(vec![chrome_tab(
            "left.hierarchy",
            "Hierarchy",
            4.0,
            4.0,
            96.0,
            24.0,
        )]),
        tabs: model_rc(vec![tab_data("left.hierarchy", "Hierarchy")]),
        ..HostSideDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_drawer_header_pointer_clicked(move |surface_key, index, tab_x, tab_width, x, y| {
                clicks
                    .borrow_mut()
                    .push((surface_key.to_string(), index, tab_x, tab_width, x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(32.0 + 4.0 + 12.0, 58.0 + 4.0 + 8.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(0.0, 58.0, 180.0, 178.0)),
        "drawer header tab selection should repaint only the owning dock region"
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [("left".to_string(), 0, 4.0, 96.0, 16.0, 12.0)]
    );
}

#[test]
fn native_host_activity_rail_press_uses_center_band_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData {
        surface_key: "left".into(),
        region_frame: host_frame(0.0, 58.0, 180.0, 178.0),
        rail_before_panel: true,
        rail_width_px: 32.0,
        rail_button_frames: model_rc(vec![control_frame("HierarchyRail", 4.0, 8.0, 24.0, 24.0)]),
        ..HostSideDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_activity_rail_pointer_clicked(move |side, x, y| {
                clicks.borrow_mut().push((side.to_string(), x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(8.0, 70.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(0.0, 58.0, 420.0, 178.0)),
        "activity rail can move drawer/document layout, so damage is the retained center band rather than full frame"
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [("left".to_string(), 8.0, 12.0)]
    );
}

#[test]
fn native_host_pointer_click_routes_host_page_tabs_with_explicit_action() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.page_chrome.tab_frames = model_rc(vec![chrome_tab(
        "HostPageTab0",
        "Workbench",
        68.0,
        29.0,
        116.0,
        28.0,
    )]);
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_host_page_pointer_clicked(move |index, close| {
                clicks.borrow_mut().push((index, close));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(80.0, 41.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(0.0, 29.0, 420.0, 231.0)),
        "host page tab activation should update presentation while repainting only page chrome, workspace, and status damage"
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [(0, false)],
        "native host page routing should emit an explicit activation receipt"
    );

    let close_result = ui.dispatch_native_primary_press_for_test(170.0, 41.0);
    assert!(close_result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [(0, false), (0, true)],
        "the committed native close frame should emit close without an editor re-hit"
    );
}

#[test]
fn native_host_pointer_click_routes_host_page_overflow_button_and_popup_rows() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.page_chrome.tab_frames = model_rc(vec![chrome_tab(
        "HostPageTab0",
        "Workbench",
        68.0,
        29.0,
        116.0,
        28.0,
    )]);
    presentation.host_scene_data.page_chrome.tabs = model_rc(vec![
        tab_data("workbench", "Workbench"),
        tab_data("assets", "Assets"),
    ]);
    presentation.host_scene_data.page_chrome.overflow_frame = host_frame(188.0, 29.0, 34.0, 28.0);
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![1];
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_host_page_pointer_clicked(move |index, close| {
                clicks.borrow_mut().push((index, close));
            });
    }

    let overflow_result = ui.dispatch_native_primary_press_for_test(198.0, 41.0);

    assert!(overflow_result.request_redraw());
    assert_eq!(clicks.borrow().as_slice(), [(-2, false)]);

    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index: -1,
            scroll_offset: 0.0,
        });
    let row_result = ui.dispatch_native_primary_press_for_test(70.0, 73.0);

    assert!(row_result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [(-2, false), (1, false)],
        "overflow popup rows should dispatch the hidden page index through the same host page callback"
    );
    assert!(
        !ui.get_host_presentation()
            .host_page_overflow_menu_state
            .open
    );
}

#[test]
fn native_host_pointer_click_routes_pane_template_button_actions() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_node_with_action(
                "ProjectPrimaryAction",
                "Button",
                "Create",
                "project.create",
                12.0,
                14.0,
                92.0,
                26.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_clicked(move |control_id, action_id| {
                clicks
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [(
            "ProjectPrimaryAction".to_string(),
            "project.create".to_string()
        )],
        "native host clicks should use template-node dispatch metadata before pane fallback routing"
    );
}

#[test]
fn native_host_pointer_click_routes_binding_only_template_buttons() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_node_with_binding(
                "SelectRoot",
                "Button",
                "Select Root",
                "HierarchyPaneBody/SelectRoot",
                12.0,
                14.0,
                92.0,
                22.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_clicked(move |control_id, action_id| {
                clicks
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [(
            "SelectRoot".to_string(),
            "HierarchyPaneBody/SelectRoot".to_string()
        )],
        "native template button hit-testing should route projected binding metadata when no literal action_id exists"
    );
}

#[test]
fn native_host_welcome_material_text_field_accepts_keyboard_input() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 340.0, 145.0),
        pane: welcome_pane_with_nodes(vec![welcome_text_node(
            "WelcomeProjectNameField",
            "ProjectNameEdited",
            "Zircon",
            20.0,
            18.0,
            240.0,
            32.0,
        )]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let changes = Rc::new(RefCell::new(Vec::new()));
    {
        let changes = changes.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_welcome_control_changed(move |control_id, value| {
                changes
                    .borrow_mut()
                    .push((control_id.to_string(), value.to_string()));
            });
    }

    let focus_result =
        ui.dispatch_native_primary_press_for_test(40.0 + 20.0 + 8.0, 58.0 + 32.0 + 18.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let backspace_result = ui.dispatch_native_backspace_for_test();
    let input_frame = host_frame(40.0 + 20.0, 58.0 + 32.0 + 18.0, 240.0, 32.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(backspace_result.request_redraw());
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame.clone()));
    assert!(!backspace_result.requires_frame_update());
    assert_eq!(backspace_result.damage_region(), Some(input_frame));
    assert_eq!(
        changes.borrow().as_slice(),
        [
            ("ProjectNameEdited".to_string(), "ZirconX".to_string()),
            ("ProjectNameEdited".to_string(), "Zircon".to_string())
        ],
        "focused Material text fields should forward keyboard edits through the welcome .zui binding route"
    );
}

#[test]
fn native_host_generic_template_text_field_routes_builtin_change_binding() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_input_node(
                "NameField",
                "Draft Cube",
                "InspectorView/NameField",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let before_snapshot = ui
        .window()
        .take_snapshot()
        .expect("pre-edit text field snapshot should render");

    let edits = Rc::new(RefCell::new(Vec::new()));
    {
        let edits = edits.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_edited(move |control_id, binding_id, value| {
                edits.borrow_mut().push((
                    control_id.to_string(),
                    binding_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    let focus_result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let after_snapshot = ui
        .window()
        .take_snapshot()
        .expect("focused text field snapshot should render the edited value");
    let input_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(!focus_result.requires_frame_update());
    assert_eq!(focus_result.damage_region(), Some(input_frame.clone()));
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame.clone()));
    assert_eq!(
        ui.get_host_presentation()
            .text_input_focus
            .value_text
            .as_str(),
        "Draft CubeX"
    );
    assert!(
        changed_pixel_count(
            after_snapshot.width(),
            before_snapshot.as_bytes(),
            after_snapshot.as_bytes(),
            input_frame.x as u32,
            input_frame.y as u32,
            input_frame.width as u32,
            input_frame.height as u32,
        ) > 0,
        "local text edit focus should repaint visible input glyphs without waiting for a full presentation rebuild"
    );
    let clear_focus_result =
        ui.dispatch_native_primary_press_for_test(60.0 + 240.0, 58.0 + 32.0 + 80.0);
    assert!(clear_focus_result.request_redraw());
    assert!(!clear_focus_result.requires_frame_update());
    assert_eq!(
        clear_focus_result.damage_region(),
        Some(input_frame.clone()),
        "clearing focus on inert pane space should repaint only the old text field"
    );
    assert!(
        !ui.get_host_presentation().text_input_focus.is_active(),
        "clicking inert pane space should clear text focus without rebuilding the full host"
    );
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "NameField".to_string(),
            "InspectorView/NameField".to_string(),
            "Draft CubeX".to_string()
        )],
        "generic template text fields should keep the shared edit binding id and changed value on the native route"
    );
}

#[test]
fn native_host_template_text_field_focus_switch_repaints_old_and_new_inputs_only() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 240));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 240.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 240.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 158.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 125.0),
        pane: pane_with_nodes(
            "Project",
            vec![
                template_input_node(
                    "NameFieldA",
                    "Draft Cube",
                    "InspectorView/NameFieldA",
                    12.0,
                    14.0,
                    160.0,
                    24.0,
                ),
                template_input_node(
                    "NameFieldB",
                    "Player",
                    "InspectorView/NameFieldB",
                    12.0,
                    48.0,
                    160.0,
                    24.0,
                ),
            ],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let first_focus =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let second_focus =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 48.0 + 8.0);
    let first_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);
    let second_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 48.0, 160.0, 24.0);
    let focus_switch_damage = host_frame(
        first_frame.x,
        first_frame.y,
        first_frame.width,
        second_frame.y + second_frame.height - first_frame.y,
    );

    assert!(first_focus.request_redraw());
    assert!(!first_focus.requires_frame_update());
    assert_eq!(first_focus.damage_region(), Some(first_frame));
    assert!(second_focus.request_redraw());
    assert!(!second_focus.requires_frame_update());
    assert_eq!(
        second_focus.damage_region(),
        Some(focus_switch_damage),
        "switching text focus should repaint the old and new input fields without a full-frame host refresh"
    );
    assert_eq!(
        ui.get_host_presentation()
            .text_input_focus
            .control_id
            .as_str(),
        "NameFieldB"
    );
}
