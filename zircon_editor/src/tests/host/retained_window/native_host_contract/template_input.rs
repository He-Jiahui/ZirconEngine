use super::support::*;

#[test]
fn native_host_asset_template_search_field_routes_asset_change_callback() {
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
        pane: assets_pane_with_nodes(vec![TemplatePaneNodeData {
            dispatch_kind: "asset".into(),
            binding_id: "AssetSurface/SearchEdited".into(),
            ..template_input_node(
                "SearchEdited",
                "cube",
                "AssetSurface/SearchEdited",
                12.0,
                14.0,
                160.0,
                24.0,
            )
        }]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let edits = Rc::new(RefCell::new(Vec::new()));
    {
        let edits = edits.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_asset_control_changed(move |source, control_id, value| {
                edits.borrow_mut().push((
                    source.to_string(),
                    control_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    let focus_result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let input_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame));
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "activity".to_string(),
            "SearchEdited".to_string(),
            "cubeX".to_string()
        )],
        ".zui asset search fields should keep their asset dispatch kind through retained projection and native text focus"
    );
}

#[test]
fn native_host_template_node_move_updates_hover_without_rebuilding_presentation() {
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
                "ProjectHoverButton",
                "Button",
                "Hover",
                "ProjectHoverAction",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let rebuild_count_after_projection = ui.presentation_rebuild_count_for_test();

    let result =
        ui.dispatch_native_pointer_move_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert!(
        !result.requires_frame_update(),
        "template hover should repaint only retained host pixels"
    );
    assert_eq!(
        result.damage_region(),
        Some(host_frame(72.0, 104.0, 160.0, 24.0)),
        "template hover should damage the hovered node instead of the full pane"
    );
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "template hover must not rebuild the projected presentation"
    );
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_control_id
            .as_str(),
        "ProjectHoverButton"
    );
    assert!(
        ui.get_host_presentation()
            .host_scene_data
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_data(0)
            .expect("hover fixture should keep its node")
            .hovered
    );

    let repeated =
        ui.dispatch_native_pointer_move_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    assert!(
        !repeated.request_redraw(),
        "same template hover target should remain on the pointer fast path"
    );

    let cleared = ui.dispatch_native_pointer_move_for_test(60.0 + 240.0, 58.0 + 32.0 + 90.0);
    assert!(cleared.request_redraw());
    assert_eq!(
        cleared.damage_region(),
        Some(host_frame(72.0, 104.0, 160.0, 24.0)),
        "leaving a template node should repaint the old hover node"
    );
    assert!(ui
        .get_pane_interaction_state()
        .hovered_template_control_id
        .is_empty());
}

#[test]
fn native_host_asset_template_buttons_route_browser_change_callback() {
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
        pane: assets_pane_with_nodes(vec![TemplatePaneNodeData {
            dispatch_kind: "asset:browser".into(),
            action_id: "workbench.asset.view_mode.set".into(),
            binding_id: "AssetSurface/SetViewMode".into(),
            value_text: "thumbnail".into(),
            ..template_node(
                "AssetBrowserViewModeThumbButton",
                "Button",
                "Thumb",
                12.0,
                14.0,
                90.0,
                24.0,
            )
        }]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let changes = Rc::new(RefCell::new(Vec::new()));
    {
        let changes = changes.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_asset_control_changed(move |source, control_id, value| {
                changes.borrow_mut().push((
                    source.to_string(),
                    control_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        changes.borrow().as_slice(),
        [(
            "browser".to_string(),
            "SetViewMode".to_string(),
            "thumbnail".to_string()
        )],
        ".zui asset browser buttons should route static value patches through the generic asset callback"
    );
}

#[test]
fn native_host_asset_browser_search_field_routes_browser_change_callback() {
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
        pane: asset_browser_pane_with_nodes(vec![TemplatePaneNodeData {
            dispatch_kind: "asset:browser".into(),
            binding_id: "AssetSurface/SearchEdited".into(),
            ..template_input_node(
                "SearchEdited",
                "mat",
                "AssetSurface/SearchEdited",
                12.0,
                14.0,
                160.0,
                24.0,
            )
        }]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let edits = Rc::new(RefCell::new(Vec::new()));
    {
        let edits = edits.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_asset_control_changed(move |source, control_id, value| {
                edits.borrow_mut().push((
                    source.to_string(),
                    control_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");

    assert!(insert_result.request_redraw());
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "browser".to_string(),
            "SearchEdited".to_string(),
            "matX".to_string()
        )],
        ".zui asset browser text fields should keep browser source through native focus edits"
    );
}

#[test]
fn native_host_generic_template_text_field_routes_commit_binding_on_enter() {
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
            vec![template_input_node_with_commit(
                "NameField",
                "Draft Cube",
                "InspectorView/NameField",
                "InspectorView/ApplyBatchButton",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

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

    ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    ui.dispatch_native_text_input_for_test("X");
    let commit_result = ui.dispatch_native_enter_for_test();

    assert!(commit_result.request_redraw());
    assert_eq!(
        edits.borrow().as_slice(),
        [
            (
                "NameField".to_string(),
                "InspectorView/NameField".to_string(),
                "Draft CubeX".to_string()
            ),
            (
                "NameField".to_string(),
                "InspectorView/ApplyBatchButton".to_string(),
                "Draft CubeX".to_string()
            )
        ],
        "Enter should dispatch the focused text field commit binding instead of reusing the edit binding"
    );
}

#[test]
fn native_host_commit_only_template_text_field_redraws_locally_until_enter() {
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
            vec![template_input_node_commit_only(
                "NameField",
                "Draft Cube",
                "InspectorView/NameFieldDraft",
                "InspectorView/ApplyBatchButton",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

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

    ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let input_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);

    assert!(insert_result.request_redraw());
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame));
    assert_eq!(
        ui.get_host_presentation()
            .text_input_focus
            .value_text
            .as_str(),
        "Draft CubeX"
    );
    assert!(
        edits.borrow().is_empty(),
        "commit-only fields should keep keystrokes local instead of dispatching asset patches"
    );

    let commit_result = ui.dispatch_native_enter_for_test();

    assert!(commit_result.request_redraw());
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "NameField".to_string(),
            "InspectorView/ApplyBatchButton".to_string(),
            "Draft CubeX".to_string()
        )],
        "commit-only fields should dispatch only the explicit commit binding on Enter"
    );
}

#[test]
fn native_host_binding_only_template_text_field_accepts_keyboard_input() {
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
            vec![template_input_node_with_binding(
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
    let input_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame));
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "NameField".to_string(),
            "InspectorView/NameField".to_string(),
            "Draft CubeX".to_string()
        )],
        "binding-only input nodes still need a single native text edit target"
    );
}

#[test]
fn native_host_resize_splitter_forwards_move_and_release_after_capture() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.resize_layer = HostResizeLayerData {
        left_splitter_frame: host_frame(120.0, 58.0, 8.0, 138.0),
        ..HostResizeLayerData::default()
    };
    ui.set_host_presentation(presentation);

    let resize_events = Rc::new(RefCell::new(Vec::new()));
    {
        let resize_events = resize_events.clone();
        ui.global::<UiHostContext>()
            .on_host_resize_pointer_event(move |kind, x, y| {
                resize_events.borrow_mut().push((kind, x, y));
            });
    }

    let press = ui.dispatch_native_primary_press_for_test(124.0, 80.0);
    let move_result = ui.dispatch_native_pointer_move_for_test(180.0, 82.0);
    let release = ui.dispatch_native_primary_release_for_test(180.0, 82.0);
    let resize_damage = host_frame(0.0, 58.0, 360.0, 138.0);

    assert!(press.request_redraw());
    assert!(move_result.request_redraw());
    assert!(release.request_redraw());
    assert!(press.requires_frame_update());
    assert!(move_result.requires_frame_update());
    assert!(release.requires_frame_update());
    assert_eq!(press.damage_region(), Some(resize_damage.clone()));
    assert_eq!(move_result.damage_region(), Some(resize_damage.clone()));
    assert_eq!(release.damage_region(), Some(resize_damage));
    assert_eq!(
        resize_events.borrow().as_slice(),
        [(0, 124.0, 80.0), (1, 180.0, 82.0), (2, 180.0, 82.0)],
        "native resize should stay captured until pointer up"
    );
}
