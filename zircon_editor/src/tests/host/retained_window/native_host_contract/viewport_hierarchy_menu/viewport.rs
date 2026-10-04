use super::super::support::*;

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
            .on_scene_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
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
            .on_scene_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
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
            .on_scene_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
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
fn native_host_game_viewport_body_never_dispatches_the_scene_callback() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    let mut game = scene_pane();
    game.kind = "Game".into();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: game,
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let scene_events = Rc::new(RefCell::new(Vec::new()));
    let game_events = Rc::new(RefCell::new(Vec::new()));
    {
        let scene_events = scene_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_scene_viewport_pointer_event(move |kind, button, _, _, _, _, _| {
                scene_events.borrow_mut().push((kind, button));
            });
        let game_events = game_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_game_viewport_pointer_event(move |kind, button, x, y, _, _, _| {
                game_events.borrow_mut().push((kind, button, x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(60.0 + 40.0, 58.0 + 32.0 + 28.0 + 12.0);

    assert!(!result.request_redraw());
    assert!(scene_events.borrow().is_empty());
    assert_eq!(game_events.borrow().as_slice(), [(0, 1, 40.0, 12.0)]);
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
            .on_scene_viewport_pointer_event(move |kind, button, x, y, delta, _, _| {
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
fn native_host_second_document_leaf_selects_its_next_viewport_product() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(720, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.document_leaves = vec![
        HostDocumentDockSurfaceData {
            surface_key: "document:left".into(),
            region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
            header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
            content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
            pane: scene_pane(),
            ..HostDocumentDockSurfaceData::default()
        },
        HostDocumentDockSurfaceData {
            surface_key: "document:right".into(),
            region_frame: host_frame(360.0, 58.0, 280.0, 138.0),
            header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
            content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
            pane: scene_pane(),
            ..HostDocumentDockSurfaceData::default()
        },
    ];
    ui.set_host_presentation(presentation);

    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_scene_viewport_pointer_event(move |kind, button, x, y, _, _, _| {
                viewport_events.borrow_mut().push((kind, button, x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(360.0 + 40.0, 58.0 + 32.0 + 28.0 + 12.0);
    assert!(!result.request_redraw());
    assert_eq!(viewport_events.borrow().as_slice(), [(0, 1, 40.0, 12.0)]);

    assert!(ui
        .global::<PaneSurfaceHostContext>()
        .set_scene_viewport_capture(
            crate::scene::viewport::RenderViewportHandle::new(3),
            crate::scene::viewport::CapturedFrame::new(1, 1, vec![0, 0, 255, 255], 9),
        ));

    let presentation = ui.get_host_presentation();
    assert!(presentation
        .viewport_images
        .for_surface("document:left", "Scene")
        .is_none());
    assert_eq!(
        presentation
            .viewport_images
            .for_surface("document:right", "Scene")
            .expect("right document leaf image")
            .resource_key,
        "viewport:3:9"
    );
}
