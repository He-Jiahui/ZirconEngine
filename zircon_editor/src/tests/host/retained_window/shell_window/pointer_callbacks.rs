use super::support::*;

#[test]
fn native_host_pointer_input_forwards_menu_callbacks_and_requests_redraw() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(320, 200));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 0.0, 0.0, 80.0, 24.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            popup_width_px: 140.0,
            popup_height_px: 96.0,
            ..HostMenuChromeMenuData::default()
        }]),
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
        let ui = ui.clone_strong();
        host.on_menu_pointer_clicked(move |x, y| {
            events.borrow_mut().push(format!("click:{x:.0},{y:.0}"));
            ui.global::<UiHostContext>()
                .set_menu_state(HostMenuStateData {
                    open_menu_index: 0,
                    ..ui.get_menu_state()
                });
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
    assert!(
        !clicked.requires_frame_update(),
        "menu clicks only mutate retained menu state and should repaint menu damage instead of the full host"
    );
    assert!(
        clicked
            .damage_region()
            .expect("menu click should return a damage region")
            .height
            > 25.0,
        "opening a menu should include popup damage, not just the top bar"
    );
    assert!(
        !scrolled.requires_frame_update(),
        "menu scroll changes local pointer state and should stay on region repaint"
    );
    assert_eq!(
        events.borrow().as_slice(),
        ["move:18,12", "click:18,12", "scroll:18,12,42"]
    );
}

#[test]
fn native_host_menu_click_unions_text_focus_damage_without_full_frame() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(320, 240));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(320.0, 240.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(320.0, 240.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 0.0, 0.0, 80.0, 24.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            popup_width_px: 140.0,
            popup_height_px: 96.0,
            ..HostMenuChromeMenuData::default()
        }]),
        ..HostMenuChromeData::default()
    };
    let mut pane = pane_with_nodes(
        "Project",
        vec![TemplatePaneNodeData {
            component_role: "input-field".into(),
            edit_action_id: "workbench.project.name.edit".into(),
            value_text: "Cube".into(),
            surface_variant: "inset".into(),
            ..template_node("NameField", "InputField", "Cube", 20.0, 12.0, 180.0, 24.0)
        }],
    );
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(320.0, 58.0));
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(0.0, 126.0, 320.0, 90.0),
        header_frame: host_frame(0.0, 0.0, 320.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 320.0, 58.0),
        pane,
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    {
        let ui = ui.clone_strong();
        ui.global::<UiHostContext>()
            .on_menu_pointer_clicked(move |_x, _y| {
                ui.global::<UiHostContext>()
                    .set_menu_state(HostMenuStateData {
                        open_menu_index: 0,
                        ..ui.get_menu_state()
                    });
            });
    }

    let input_frame = host_frame(20.0, 170.0, 180.0, 24.0);
    let focus_result = ui.dispatch_native_primary_press_for_test(28.0, 178.0);
    assert!(focus_result.request_redraw());
    assert!(!focus_result.requires_frame_update());
    assert_eq!(focus_result.damage_region(), Some(input_frame.clone()));

    let clicked = ui.dispatch_native_primary_press_for_test(18.0, 12.0);
    let damage = clicked
        .damage_region()
        .expect("menu click should return regional damage");

    assert!(clicked.request_redraw());
    assert!(
        !clicked.requires_frame_update(),
        "menu click after input focus clear should still avoid a full host frame update"
    );
    assert!(
        damage.x <= input_frame.x
            && damage.y <= input_frame.y
            && damage.x + damage.width >= input_frame.x + input_frame.width
            && damage.y + damage.height >= input_frame.y + input_frame.height,
        "menu click damage must include the old text input focus frame"
    );
    assert!(
        damage.height > 160.0,
        "menu damage should be unioned with the lower input focus damage"
    );
}

#[test]
fn native_host_frame_request_invokes_installed_frame_callback_before_presenting() {
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
