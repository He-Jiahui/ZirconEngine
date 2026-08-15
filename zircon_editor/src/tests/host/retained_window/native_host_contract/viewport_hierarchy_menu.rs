use super::support::*;

#[test]
fn native_host_pointer_click_routes_viewport_toolbar_buttons_before_viewport_body() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(720, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 620.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 620.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 620.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    let tool_frame = viewport_toolbar_control_frame(&presentation, "tool.move");
    ui.set_host_presentation(presentation);

    let toolbar_clicks = Rc::new(RefCell::new(Vec::new()));
    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let toolbar_clicks = toolbar_clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_toolbar_pointer_clicked(
                move |surface_key, point_x, point_y, width, height| {
                    toolbar_clicks.borrow_mut().push((
                        surface_key.to_string(),
                        point_x,
                        point_y,
                        width,
                        height,
                    ));
                },
            );
    }
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(
        60.0 + tool_frame.x + tool_frame.width * 0.5,
        58.0 + 32.0 + tool_frame.y + tool_frame.height * 0.5,
    );

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(60.0, 90.0, 620.0, 28.0))
    );
    assert_eq!(viewport_events.borrow().as_slice(), []);
    let clicks = toolbar_clicks.borrow();
    assert_eq!(clicks.len(), 1);
    assert_eq!(clicks[0].0, "document");
    assert_eq!(clicks[0].1, tool_frame.x + tool_frame.width * 0.5);
    assert_eq!(clicks[0].2, tool_frame.y + tool_frame.height * 0.5);
    assert_eq!(clicks[0].3, 620.0);
    assert_eq!(clicks[0].4, 28.0);
}

#[test]
fn native_host_viewport_toolbar_only_dispatches_primary_press() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(720, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 620.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 620.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 620.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    let display_frame = viewport_toolbar_control_frame(&presentation, "display.cycle");
    ui.set_host_presentation(presentation);

    let toolbar_clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let toolbar_clicks = toolbar_clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_toolbar_pointer_clicked(
                move |surface_key, _point_x, _point_y, _width, _height| {
                    toolbar_clicks.borrow_mut().push(surface_key.to_string());
                },
            );
    }

    let display_x = 60.0 + display_frame.x + display_frame.width * 0.5;
    let toolbar_y = 58.0 + 32.0 + display_frame.y + display_frame.height * 0.5;
    let press = ui.dispatch_native_primary_press_for_test(display_x, toolbar_y);
    let release = ui.dispatch_native_primary_release_for_test(display_x, toolbar_y);
    let secondary = ui.dispatch_native_secondary_press_for_test(display_x, toolbar_y);
    let middle = ui.dispatch_native_middle_press_for_test(display_x, toolbar_y);

    assert!(press.request_redraw());
    assert!(press.requires_frame_update());
    assert_eq!(
        press.damage_region(),
        Some(host_frame(60.0, 90.0, 620.0, 28.0))
    );
    assert!(!release.request_redraw());
    assert!(!secondary.request_redraw());
    assert!(!middle.request_redraw());
    assert_eq!(toolbar_clicks.borrow().as_slice(), ["document"]);
}

#[test]
fn native_host_pointer_click_routes_late_viewport_toolbar_controls() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(900, 240));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(900.0, 240.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(900.0, 240.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 800.0, 158.0),
        header_frame: host_frame(0.0, 0.0, 800.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 800.0, 125.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    let frame_selection_frame = viewport_toolbar_control_frame(&presentation, "frame.selection");
    ui.set_host_presentation(presentation);

    let toolbar_clicks = Rc::new(RefCell::new(Vec::new()));
    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let toolbar_clicks = toolbar_clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_toolbar_pointer_clicked(
                move |surface_key, point_x, _point_y, width, _height| {
                    toolbar_clicks
                        .borrow_mut()
                        .push((surface_key.to_string(), width, point_x));
                },
            );
    }
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let frame_selection_x = 60.0 + frame_selection_frame.x + frame_selection_frame.width * 0.5;
    let toolbar_y = 58.0 + 32.0 + frame_selection_frame.y + frame_selection_frame.height * 0.5;
    let result = ui.dispatch_native_primary_press_for_test(frame_selection_x, toolbar_y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(0.0, 58.0, 900.0, 182.0)),
        "viewport commands that can move the camera or status should repaint center band and status, not the full host"
    );
    assert_eq!(viewport_events.borrow().as_slice(), []);
    assert_eq!(
        toolbar_clicks.borrow().as_slice(),
        [(
            "document".to_string(),
            800.0,
            frame_selection_frame.x + frame_selection_frame.width * 0.5,
        )]
    );
}

#[test]
fn native_host_pointer_move_routes_viewport_without_native_repaint() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(60.0 + 40.0, 58.0 + 32.0 + 28.0 + 12.0);

    assert!(
        !result.request_redraw(),
        "viewport moves update runtime input state; native repaint waits for the next viewport image"
    );
    assert_eq!(
        viewport_events.borrow().as_slice(),
        [(1, 0, 40.0, 12.0, 0.0)],
        "viewport move facts should still reach the shared pointer bridge"
    );
}

#[test]
fn native_host_viewport_button_and_scroll_wait_for_viewport_image_repaint() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let rebuild_count_after_projection = ui.presentation_rebuild_count_for_test();

    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let x = 60.0 + 40.0;
    let y = 58.0 + 32.0 + 28.0 + 12.0;
    let press = ui.dispatch_native_primary_press_for_test(x, y);
    let release = ui.dispatch_native_primary_release_for_test(x, y);
    let scroll = ui.dispatch_native_pointer_scroll_for_test(x, y, -120.0);

    assert!(
        !press.request_redraw(),
        "viewport press updates runtime input; native repaint waits for the next viewport image"
    );
    assert!(
        !release.request_redraw(),
        "viewport release should not force a stale native repaint"
    );
    assert!(
        !scroll.request_redraw(),
        "viewport scroll should not repaint the old viewport image before the renderer updates it"
    );
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "viewport pointer events must not rebuild projected presentation state"
    );
    assert_eq!(
        viewport_events.borrow().as_slice(),
        [
            (0, 1, 40.0, 12.0, 0.0),
            (2, 1, 40.0, 12.0, 0.0),
            (3, 0, 40.0, 12.0, -120.0),
        ],
        "viewport press/release/scroll facts should still reach the shared pointer bridge"
    );
}

#[test]
fn native_host_hierarchy_move_updates_visible_hover_state() {
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
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: hierarchy_pane(vec![
            scene_node("entity://root", "Root", 0, false),
            scene_node("entity://child", "Child", 1, false),
        ]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let before = ui
        .window()
        .take_snapshot()
        .expect("pre-hover hierarchy snapshot should render");
    {
        let ui = ui.clone_strong();
        ui.global::<PaneSurfaceHostContext>()
            .on_hierarchy_pointer_moved(move |_x, _y, _width, _height| {
                ui.global::<PaneSurfaceHostContext>()
                    .set_hovered_hierarchy_index(1);
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 42.0);
    let after = ui
        .window()
        .take_snapshot()
        .expect("post-hover hierarchy snapshot should render");

    assert!(result.request_redraw());
    assert!(
        !result.requires_frame_update(),
        "native hover should repaint the pane region without forcing a full frame update"
    );
    assert_eq!(
        result.damage_region(),
        Some(host_frame(28.0, 96.0, 284.0, 22.0)),
        "hierarchy hover should damage the changed row instead of the full host frame"
    );
    assert!(
        changed_pixel_count(
            after.width(),
            before.as_bytes(),
            after.as_bytes(),
            28,
            94,
            284,
            26,
        ) > 80,
        "native hierarchy hover state should be visible in the rust-owned host painter"
    );
    let repeated = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 42.0);
    assert!(
        !repeated.request_redraw(),
        "repeating the same hierarchy hover target should be a pointer fast path"
    );
}

#[test]
fn native_host_hierarchy_move_prefers_native_hover_when_template_node_overlaps() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: hierarchy_pane_with_template_nodes(
            vec![
                scene_node("entity://root", "Root", 0, false),
                scene_node("entity://child", "Child", 1, false),
            ],
            vec![template_node_with_action(
                "HierarchyTemplateOverlay",
                "Button",
                "Overlay",
                "OverlayAction",
                0.0,
                0.0,
                300.0,
                124.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let moves = Rc::new(RefCell::new(Vec::new()));
    {
        let ui = ui.clone_strong();
        let moves = moves.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_hierarchy_pointer_moved(move |x, y, width, height| {
                moves.borrow_mut().push((x, y, width, height));
                ui.global::<PaneSurfaceHostContext>()
                    .set_hovered_hierarchy_index(1);
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 42.0);

    assert!(result.request_redraw());
    assert_eq!(
        moves.borrow().as_slice(),
        [(20.0, 42.0, 300.0, 124.0)],
        "template hit surfaces must not swallow native hierarchy hover routing"
    );
    assert_eq!(
        result.damage_region(),
        Some(host_frame(28.0, 96.0, 284.0, 22.0)),
        "hierarchy hover should still use row-local damage under template-backed panes"
    );
}

#[test]
fn native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation() {
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
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: hierarchy_pane(vec![
            scene_node("entity://root", "Root", 0, false),
            scene_node("entity://child", "Child", 1, false),
        ]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let rebuild_count_after_projection = ui.presentation_rebuild_count_for_test();
    {
        let ui = ui.clone_strong();
        ui.global::<PaneSurfaceHostContext>()
            .on_hierarchy_pointer_moved(move |_x, _y, _width, _height| {
                ui.global::<PaneSurfaceHostContext>()
                    .set_hovered_hierarchy_index(1);
            });
    }

    let hover_x = 20.0 + 20.0;
    let hover_y = 40.0 + 25.0 + 42.0;
    let first = ui.dispatch_native_pointer_move_for_test(hover_x, hover_y);

    assert!(first.request_redraw());
    assert!(
        !first.requires_frame_update(),
        "first hierarchy hover should use local paint damage"
    );
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "pointer-only hover must not rebuild the projected presentation"
    );

    for _ in 0..100 {
        let repeated = ui.dispatch_native_pointer_move_for_test(hover_x, hover_y);
        assert!(
            !repeated.request_redraw(),
            "same-target hierarchy hover should stay on the pointer fast path"
        );
        assert!(
            !repeated.requires_frame_update(),
            "same-target hierarchy hover must not request a full frame update"
        );
    }
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "100 same-target hover moves must not rebuild presentation state"
    );
}

#[test]
fn native_host_hierarchy_press_uses_pane_center_status_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: hierarchy_pane(vec![
            scene_node("entity://root", "Root", 0, false),
            scene_node("entity://child", "Child", 1, false),
        ]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_hierarchy_pointer_clicked(move |x, y, width, height| {
                clicks.borrow_mut().push((x, y, width, height));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(60.0 + 20.0, 58.0 + 32.0 + 42.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(0.0, 58.0, 360.0, 162.0)),
        "pane press callbacks should refresh presentation while repainting center/status damage, not the full native window"
    );
    assert_eq!(clicks.borrow().as_slice(), [(20.0, 42.0, 280.0, 105.0)]);
}

#[test]
fn native_host_asset_tree_move_updates_visible_hover_state() {
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
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: asset_tree_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let before = ui
        .window()
        .take_snapshot()
        .expect("pre-hover asset tree snapshot should render");
    {
        let ui = ui.clone_strong();
        ui.global::<PaneSurfaceHostContext>()
            .on_asset_tree_pointer_moved(move |_mode, _x, _y, _width, _height| {
                ui.global::<PaneSurfaceHostContext>()
                    .set_activity_asset_tree_hovered_index(0);
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 57.0 + 12.0);
    let after = ui
        .window()
        .take_snapshot()
        .expect("post-hover asset tree snapshot should render");

    assert!(result.request_redraw());
    assert!(
        !result.requires_frame_update(),
        "native asset hover should repaint the pane region without forcing a full frame update"
    );
    assert!(
        changed_pixel_count(
            after.width(),
            before.as_bytes(),
            after.as_bytes(),
            28,
            122,
            220,
            28,
        ) > 80,
        "native asset tree hover state should be visible in the rust-owned host painter"
    );
    let repeated = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 57.0 + 12.0);
    assert!(
        !repeated.request_redraw(),
        "repeating the same asset-tree hover target should not repaint"
    );
}

#[test]
fn rust_owned_host_painter_draws_open_menu_popup_above_pane_surfaces() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut closed = ui.get_host_presentation();
    closed.host_layout = host_window_layout_for_test(360.0, 220.0);
    closed.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    closed.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "File".into(),
            popup_width_px: 144.0,
            popup_height_px: 66.0,
            items: model_rc(vec![
                HostMenuChromeItemData {
                    label: "Open".into(),
                    action_id: "workbench.project.open".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Reset Layout".into(),
                    action_id: "workbench.layout.reset".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
            ]),
            popup_nodes: model_rc(vec![
                template_node("MenuPopupPanel", "Panel", "", 0.0, 0.0, 144.0, 66.0),
                template_node("MenuPopupItemRow0", "Panel", "Open", 6.0, 6.0, 132.0, 26.0),
                template_node(
                    "MenuPopupItemRow1",
                    "Panel",
                    "Reset",
                    6.0,
                    34.0,
                    132.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    };
    closed.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    closed.menu_state = HostMenuStateData {
        open_menu_index: -1,
        ..HostMenuStateData::default()
    };
    ui.set_host_presentation(closed.clone());
    let closed_snapshot = ui
        .window()
        .take_snapshot()
        .expect("closed menu snapshot should render");

    let mut open = closed;
    open.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let open_menu_state = open.menu_state.clone();
    ui.set_host_presentation(open);
    ui.global::<UiHostContext>().set_menu_state(open_menu_state);
    let open_snapshot = ui
        .window()
        .take_snapshot()
        .expect("open menu snapshot should render");

    assert!(
        changed_pixel_count(
            open_snapshot.width(),
            closed_snapshot.as_bytes(),
            open_snapshot.as_bytes(),
            8,
            27,
            144,
            66,
        ) > 200,
        "open menu popup should paint over the document/viewport surface below the menu bar"
    );
}

#[test]
fn rust_owned_host_painter_draws_open_nested_menu_popup() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "Tools".into(),
            popup_width_px: 144.0,
            popup_height_px: 38.0,
            items: model_rc(vec![HostMenuChromeItemData {
                label: "Weather".into(),
                shortcut: ">".into(),
                enabled: true,
                children: model_rc(vec![HostMenuChromeItemData {
                    label: "Refresh Clouds".into(),
                    action_id: "weather.cloud_layer.refresh".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                }]),
                ..HostMenuChromeItemData::default()
            }]),
            popup_nodes: model_rc(vec![
                template_node("MenuPopupPanel", "Panel", "", 0.0, 0.0, 144.0, 38.0),
                template_node(
                    "MenuPopupItemRow0",
                    "Panel",
                    "Weather",
                    6.0,
                    6.0,
                    132.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    };
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let root_menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation.clone());
    ui.global::<UiHostContext>().set_menu_state(root_menu_state);
    let root_only = ui
        .window()
        .take_snapshot()
        .expect("root menu snapshot should render");

    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        open_submenu_path: vec![0],
        hovered_menu_item_path: vec![0, 0],
        hovered_menu_item_index: 1,
        ..HostMenuStateData::default()
    };
    let nested_menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>()
        .set_menu_state(nested_menu_state);
    let nested = ui
        .window()
        .take_snapshot()
        .expect("nested menu snapshot should render");

    assert!(
        changed_pixel_count(
            nested.width(),
            root_only.as_bytes(),
            nested.as_bytes(),
            148,
            33,
            150,
            42,
        ) > 140,
        "opening a submenu branch should paint a visible child popup beside the root menu"
    );
}
