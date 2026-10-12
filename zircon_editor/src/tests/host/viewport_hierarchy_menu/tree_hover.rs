use super::super::support::*;

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
