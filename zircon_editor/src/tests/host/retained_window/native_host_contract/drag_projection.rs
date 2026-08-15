use super::support::*;

#[test]
fn native_host_welcome_material_button_routes_welcome_callback() {
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
        pane: welcome_pane_with_nodes(vec![welcome_button_node(
            "WelcomeCreateProjectButton",
            "CreateProject",
            "Create Project",
            20.0,
            64.0,
            132.0,
            30.0,
        )]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_welcome_control_clicked(move |control_id| {
                clicks.borrow_mut().push(control_id.to_string());
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(40.0 + 20.0 + 8.0, 58.0 + 32.0 + 64.0 + 8.0);
    let release =
        ui.dispatch_native_primary_release_for_test(40.0 + 20.0 + 8.0, 58.0 + 32.0 + 64.0 + 8.0);

    assert!(result.request_redraw());
    assert!(release.request_redraw());
    assert!(
        !release.requires_frame_update(),
        "button release only clears local pressed state and should not request a full frame update"
    );
    assert_eq!(
        release.damage_region(),
        Some(host_frame(40.0, 90.0, 340.0, 145.0))
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        ["CreateProject".to_string()],
        "welcome Material buttons should route to the welcome surface command bridge"
    );
}

#[test]
fn native_host_document_tab_drag_releases_capture_and_forwards_drop() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
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

    let drag_events = Rc::new(RefCell::new(Vec::new()));
    {
        let drag_events = drag_events.clone();
        ui.global::<UiHostContext>()
            .on_host_drag_pointer_event(move |kind, x, y| {
                drag_events.borrow_mut().push((kind, x, y));
            });
    }

    ui.dispatch_native_primary_press_for_test(40.0 + 24.0, 58.0 + 12.0);
    ui.dispatch_native_pointer_move_for_test(40.0 + 132.0, 58.0 + 74.0);
    let host = ui.global::<UiHostContext>();
    let mut active_drag_state = host.get_drag_state();
    active_drag_state.active_drag_target_group = "document".into();
    host.set_drag_state(active_drag_state);
    let release = ui.dispatch_native_primary_release_for_test(40.0 + 132.0, 58.0 + 74.0);

    let drag_state = ui.global::<UiHostContext>().get_drag_state();
    assert!(release.request_redraw());
    assert!(
        release.requires_frame_update(),
        "same-dock tab release can refresh presentation while preserving regional damage"
    );
    assert_eq!(
        release.damage_region(),
        Some(host_frame(0.0, 58.0, 420.0, 202.0))
    );
    assert_eq!(
        drag_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (2, 172.0, 132.0)],
        "native document-tab drags should enter and leave the shared Slate drag path"
    );
    assert!(
        drag_state.drag_tab_id.is_empty() && !drag_state.drag_active,
        "drag capture must be cleared on primary release so native repaint cannot leave stale drag state behind"
    );
}

#[test]
fn native_host_document_tab_drag_cross_dock_release_uses_center_status_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData {
        surface_key: "left".into(),
        region_frame: host_frame(0.0, 58.0, 120.0, 178.0),
        content_frame: host_frame(0.0, 31.0, 120.0, 147.0),
        pane: hierarchy_pane(vec![scene_node("entity://root", "Root", 0, false)]),
        ..HostSideDockSurfaceData::default()
    };
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(140.0, 58.0, 240.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 240.0, 31.0),
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

    ui.dispatch_native_primary_press_for_test(140.0 + 24.0, 58.0 + 12.0);
    ui.dispatch_native_pointer_move_for_test(140.0 + 132.0, 58.0 + 74.0);
    let host = ui.global::<UiHostContext>();
    let mut active_drag_state = host.get_drag_state();
    active_drag_state.active_drag_target_group = "left".into();
    host.set_drag_state(active_drag_state);
    let release = ui.dispatch_native_primary_release_for_test(40.0, 120.0);

    assert!(release.request_redraw());
    assert!(release.requires_frame_update());
    assert_eq!(
        release.damage_region(),
        Some(host_frame(0.0, 58.0, 420.0, 202.0)),
        "cross-dock tab release should repaint center/status damage without falling back to a full native frame"
    );
}

#[test]
fn native_host_document_tab_drag_document_edge_release_uses_center_status_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
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

    ui.dispatch_native_primary_press_for_test(40.0 + 24.0, 58.0 + 12.0);
    ui.dispatch_native_pointer_move_for_test(40.0 + 132.0, 58.0 + 74.0);
    let host = ui.global::<UiHostContext>();
    let mut active_drag_state = host.get_drag_state();
    active_drag_state.active_drag_target_group = "document-left".into();
    host.set_drag_state(active_drag_state);
    let release = ui.dispatch_native_primary_release_for_test(40.0, 120.0);

    assert!(release.request_redraw());
    assert!(release.requires_frame_update());
    assert_eq!(
        release.damage_region(),
        Some(host_frame(0.0, 58.0, 420.0, 202.0)),
        "document-edge tab split should repaint center/status damage without falling back to a full native frame"
    );
}

#[test]
fn native_host_document_tab_drag_floating_window_release_uses_floating_center_status_damage() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 58.0, 240.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 240.0, 31.0),
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
    presentation.host_scene_data.floating_layer.floating_windows =
        model_rc(vec![FloatingWindowData {
            window_id: "floating-a".into(),
            frame: host_frame(300.0, 40.0, 80.0, 50.0),
            header_frame: host_frame(0.0, 0.0, 80.0, 24.0),
            target_group: "floating-window/floating-a".into(),
            active_pane: scene_pane(),
            ..FloatingWindowData::default()
        }]);
    ui.set_host_presentation(presentation);

    ui.dispatch_native_primary_press_for_test(40.0 + 24.0, 58.0 + 12.0);
    ui.dispatch_native_pointer_move_for_test(40.0 + 132.0, 58.0 + 74.0);
    let host = ui.global::<UiHostContext>();
    let mut active_drag_state = host.get_drag_state();
    active_drag_state.active_drag_target_group = "floating-window/floating-a".into();
    host.set_drag_state(active_drag_state);
    let release = ui.dispatch_native_primary_release_for_test(320.0, 58.0);

    assert!(release.request_redraw());
    assert!(release.requires_frame_update());
    assert_eq!(
        release.damage_region(),
        Some(host_frame(0.0, 40.0, 420.0, 220.0)),
        "floating-window tab drop should repaint the floating target plus center/status damage instead of the full native frame"
    );
}

#[test]
fn native_host_pointer_click_routes_projected_material_showcase_button() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates()
        .expect("builtin host templates should load for showcase native host test");
    let showcase_pane = component_showcase_pane_with_runtime_projection(&runtime, 900.0, 760.0);
    let button = template_node_by_control_id(&showcase_pane, "ButtonDemo");
    assert_eq!(button.dispatch_kind.as_str(), "showcase");
    assert_eq!(
        button.action_id.as_str(),
        "UiComponentShowcase/ButtonCommit"
    );
    assert!(
        button.frame.width > 0.0 && button.frame.height > 0.0,
        "projected Material ButtonDemo should carry a concrete arranged frame"
    );
    let hit_frame_node = showcase_pane
        .body_surface_frame
        .as_ref()
        .and_then(|frame| {
            frame
                .arranged_tree
                .nodes
                .iter()
                .find(|node| node.control_id.as_deref() == Some("ButtonDemo"))
        })
        .expect("ButtonDemo should be present in the retained hit-test surface frame");
    assert!(hit_frame_node.enabled && hit_frame_node.clickable);
    assert_eq!(hit_frame_node.frame.x, button.frame.x);
    assert_eq!(hit_frame_node.frame.y, button.frame.y);
    assert_eq!(hit_frame_node.frame.width, button.frame.width);
    assert_eq!(hit_frame_node.frame.height, button.frame.height);

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(980, 840));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(980.0, 840.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(980.0, 840.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 52.0, 920.0, 760.0),
        header_frame: host_frame(0.0, 0.0, 920.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 900.0, 720.0),
        pane: showcase_pane.clone(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let activations = Rc::new(RefCell::new(Vec::new()));
    {
        let activations = activations.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_component_showcase_control_activated(move |control_id, action_id| {
                activations
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(
        40.0 + button.frame.x + button.frame.width * 0.5,
        52.0 + 32.0 + button.frame.y + button.frame.height * 0.5,
    );

    assert!(result.request_redraw());
    assert_eq!(
        activations.borrow().as_slice(),
        [(
            "ButtonDemo".to_string(),
            "UiComponentShowcase/ButtonCommit".to_string()
        )],
        "native host should route real .zui Material component hits through showcase dispatch metadata"
    );
}

#[test]
fn native_host_pointer_click_ignores_disabled_projected_template_button() {
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
            vec![TemplatePaneNodeData {
                disabled: true,
                ..template_node_with_action(
                    "DisabledButton",
                    "Button",
                    "Disabled",
                    "Disabled.Action",
                    12.0,
                    14.0,
                    92.0,
                    22.0,
                )
            }],
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

    assert!(!result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [],
        "disabled template Buttons must not enter retained-host pointer dispatch"
    );
}

#[test]
fn native_host_pointer_click_ignores_template_buttons_without_dispatch_metadata() {
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
            vec![template_node(
                "MissingDispatch",
                "Button",
                "Decorative",
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

    assert!(
        !result.request_redraw(),
        "decorative template nodes without dispatch metadata should not consume CPU with a native repaint"
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [],
        "visual buttons without action, binding, or dispatch kind must not consume native clicks as empty actions"
    );
}

#[test]
fn native_host_pointer_click_routes_componentized_workbench_window_nodes() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.workbench_window_nodes = model_rc(vec![TemplatePaneNodeData {
        dispatch_kind: "workbench".into(),
        ..template_node_with_binding(
            "WorkbenchToolScale",
            "Button",
            "Scale",
            "Tool/Scale",
            24.0,
            18.0,
            88.0,
            28.0,
        )
    }]);
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

    let result = ui.dispatch_native_primary_press_for_test(32.0, 28.0);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        result.damage_region(),
        Some(host_frame(24.0, 18.0, 88.0, 28.0))
    );
    assert_eq!(
        clicks.borrow().as_slice(),
        [("WorkbenchToolScale".to_string(), "Tool/Scale".to_string())],
        "native host should route full-window componentized workbench nodes through surface controls"
    );
}

#[test]
fn native_host_pointer_hover_updates_componentized_workbench_window_nodes() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.workbench_window_nodes = model_rc(vec![TemplatePaneNodeData {
        dispatch_kind: "workbench".into(),
        ..template_node_with_binding(
            "WorkbenchPrimaryButton",
            "Button",
            "Primary",
            "ComponentLab/PrimaryButton",
            24.0,
            18.0,
            88.0,
            28.0,
        )
    }]);
    ui.set_host_presentation(presentation);

    let result = ui.dispatch_native_pointer_move_for_test(32.0, 28.0);

    assert!(result.request_redraw());
    let presentation = ui.get_host_presentation();
    let hovered = presentation
        .workbench_window_nodes
        .row_data(0)
        .expect("workbench test node should remain projected");
    assert!(hovered.hovered);
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_control_id
            .as_str(),
        "WorkbenchPrimaryButton"
    );
}
