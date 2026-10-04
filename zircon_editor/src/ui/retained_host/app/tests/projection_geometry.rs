use super::support::*;

#[test]
fn root_viewport_toolbar_pointer_click_uses_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_viewport_toolbar_projection");
    let baseline = harness.journal_len();

    pane_surface_host(&harness.root_ui).invoke_viewport_toolbar_pointer_clicked(
        "editor.scene#1".into(),
        300.0,
        10.0,
        1280.0,
        28.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Viewport(EditorViewportEvent::SetDisplayMode {
            mode: DisplayMode::WireOverlay,
        })]
    );
}

#[test]
fn root_viewport_toolbar_repeated_click_reuses_published_layout_authority() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_viewport_toolbar_cached_click");
    let baseline = harness.journal_len();
    let recomputes_before = harness
        .host
        .borrow()
        .viewport_toolbar_bridge
        .layout_recompute_count();

    for _ in 0..2 {
        pane_surface_host(&harness.root_ui).invoke_viewport_toolbar_pointer_clicked(
            "editor.scene#1".into(),
            300.0,
            10.0,
            1280.0,
            28.0,
        );
    }

    assert_eq!(harness.delta_events_since(baseline).len(), 2);
    assert_eq!(
        harness
            .host
            .borrow()
            .viewport_toolbar_bridge
            .layout_recompute_count(),
        recomputes_before,
        "click dispatch must consume the published surface frame without rebuilding template layout"
    );
}

#[test]
fn root_viewport_toolbar_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_viewport_toolbar_projection_width");
    let (point_x, point_y) = {
        let mut host = harness.host.borrow_mut();
        let geometry = host
            .shell_geometry
            .as_mut()
            .expect("root host should have computed shell geometry");
        geometry
            .region_frames
            .insert(ShellRegionId::Left, ShellFrame::default());
        geometry
            .region_frames
            .insert(ShellRegionId::Right, ShellFrame::default());
        geometry
            .region_frames
            .insert(ShellRegionId::Bottom, ShellFrame::default());
        let document = geometry.region_frame(ShellRegionId::Document);
        geometry.region_frames.insert(
            ShellRegionId::Document,
            ShellFrame::new(document.x, document.y, 800.0, document.height),
        );

        let surface_size = host.viewport_toolbar_surface_size("editor.scene#1");
        assert!(
            surface_size.width > 1000.0,
            "shared projection width should outrank stale document geometry"
        );
        host.viewport_toolbar_bridge
            .recompute_layout(surface_size)
            .expect("viewport toolbar projection should recompute");
        let control_frame = host
            .viewport_toolbar_bridge
            .control_frame_for_control("AlignView")
            .expect("align.neg_z should map to a projected control frame");
        (
            control_frame.x + control_frame.width * 0.75,
            control_frame.y + control_frame.height * 0.5,
        )
    };
    let baseline = harness.journal_len();

    pane_surface_host(&harness.root_ui).invoke_viewport_toolbar_pointer_clicked(
        "editor.scene#1".into(),
        point_x,
        point_y,
        1280.0,
        28.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Viewport(EditorViewportEvent::AlignView {
            orientation: ViewOrientation::NegZ,
        })]
    );
}

#[test]
fn root_viewport_toolbar_surface_size_prefers_shared_projection_width_when_document_geometry_is_oversized(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_viewport_toolbar_projection_oversized");
    let mut host = harness.host.borrow_mut();
    let expected_width = host
        .template_bridge
        .control_frame("PaneSurfaceRoot")
        .expect("pane surface root should map to a projected control frame")
        .width;
    let geometry = host
        .shell_geometry
        .as_mut()
        .expect("root host should have computed shell geometry");
    geometry
        .region_frames
        .insert(ShellRegionId::Left, ShellFrame::default());
    geometry
        .region_frames
        .insert(ShellRegionId::Right, ShellFrame::default());
    geometry
        .region_frames
        .insert(ShellRegionId::Bottom, ShellFrame::default());
    let document = geometry.region_frame(ShellRegionId::Document);
    geometry.region_frames.insert(
        ShellRegionId::Document,
        ShellFrame::new(
            document.x,
            document.y,
            expected_width + 480.0,
            document.height,
        ),
    );

    assert_eq!(
        host.viewport_toolbar_surface_size("editor.scene#1"),
        UiSize::new(expected_width, 28.0),
        "shared projection width should remain authoritative even when legacy document geometry is wider"
    );
}

#[test]
fn root_document_tab_native_receipt_ignores_stale_mirror_geometry() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_document_tab_projection_width");
    harness.activate_workbench_page();
    let expected_instance = ViewInstanceId::new("editor.scene#1");
    let (tab_index, tab_x, point_x, point_y, tab_width) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(
            &crate::core::commands::EditorCommandRegistry::default_workbench(),
            &chrome,
        );
        let tab_index = model
            .document_tabs
            .iter()
            .position(|tab| tab.instance_id == expected_instance)
            .expect("scene view should exist in document tabs");
        let shared_tabs_frame = host
            .template_bridge
            .control_frame("DocumentTabsRoot")
            .expect("document tabs root should map to a projected control frame");
        assert!(
            shared_tabs_frame.width > 1000.0,
            "shared projection width should outrank stale document geometry"
        );
        {
            let geometry = host
                .shell_geometry
                .as_mut()
                .expect("root host should have computed shell geometry");
            geometry
                .region_frames
                .insert(ShellRegionId::Left, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Right, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Bottom, ShellFrame::default());
            let document = geometry.region_frame(ShellRegionId::Document);
            geometry.region_frames.insert(
                ShellRegionId::Document,
                ShellFrame::new(document.x, document.y, 800.0, document.height),
            );
            host.sync_document_tab_pointer_layout(&model);
        }

        let tab_width = 140.0;
        let tab_x = shared_tabs_frame.width - tab_width - 24.0;
        (tab_index as i32, tab_x, tab_x + 32.0, 14.0, tab_width)
    };
    let baseline = harness.journal_len();

    host_context(&harness.root_ui).invoke_document_tab_pointer_clicked(
        "document".into(),
        tab_index,
        tab_x,
        tab_width,
        point_x,
        point_y,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new(expected_instance.0.clone()),
        })]
    );
    assert!(
        !harness
            .host
            .borrow()
            .runtime
            .editor_snapshot()
            .status_line
            .contains("Unknown document tab surface"),
        "root document tab callbacks should use the same surface key registered by the pointer bridge"
    );
}

#[test]
fn root_host_page_pointer_click_uses_shared_projection_tab_slot() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_host_page_projection_width");
    {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(
            &crate::core::commands::EditorCommandRegistry::default_workbench(),
            &chrome,
        );
        let shared_shell_frame = host
            .template_bridge
            .control_frame("UiHostWindowRoot")
            .expect("workbench shell root should map to a projected control frame");
        assert!(
            shared_shell_frame.width > 1000.0,
            "shared shell projection width should outrank host-page metric estimates"
        );
        host.sync_host_page_pointer_layout(&model);
    }
    let baseline = harness.journal_len();

    host_context(&harness.root_ui).invoke_host_page_pointer_clicked(0, false);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::ActivateMainPage {
            page_id: EventMainPageId::workbench(),
        })]
    );
}

#[test]
fn root_activity_rail_pointer_click_prefers_shared_projection_surface_when_left_region_geometry_is_stale(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_activity_rail_projection_width");
    harness.activate_workbench_page();
    let (point_x, point_y) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(
            &crate::core::commands::EditorCommandRegistry::default_workbench(),
            &chrome,
        );
        let shared_activity_rail = host
            .template_bridge
            .control_frame("ActivityRailRoot")
            .expect("activity rail root should map to a projected control frame");
        assert!(
            shared_activity_rail.width > 0.0,
            "shared projection activity rail should exist"
        );
        {
            let geometry = host
                .shell_geometry
                .as_mut()
                .expect("root host should have computed shell geometry");
            geometry
                .region_frames
                .insert(ShellRegionId::Left, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Right, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Bottom, ShellFrame::default());
            host.sync_activity_rail_pointer_layout(&model);
        }

        (shared_activity_rail.width * 0.5, 20.0)
    };
    let baseline = harness.journal_len();

    host_context(&harness.root_ui).invoke_activity_rail_pointer_clicked(
        "left".into(),
        point_x,
        point_y,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::SetDrawerMode {
            slot: EventActivityDrawerSlot::LeftTop,
            mode: EventActivityDrawerMode::Collapsed,
        })]
    );
}

#[test]
fn root_resize_capture_prefers_workbench_left_drawer_shell_extent_over_stale_region_geometry() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_resize_projection_extent");
    harness.activate_workbench_page();

    let mut host = harness.host.borrow_mut();
    let workbench_layout_frames = host.workbench_window_bridge.layout_frames();
    let expected_width = workbench_layout_frames
        .left_drawer_shell_frame
        .expect("left drawer shell root should map to a Workbench layout frame")
        .width;
    let splitter = workbench_layout_frames
        .left_resize_splitter_frame
        .expect("Workbench layout frames should expose the left resize splitter");
    assert!(
        splitter.width > 0.0 && splitter.height > 0.0,
        "Workbench layout frames should expose the left resize splitter"
    );
    let geometry = host
        .shell_geometry
        .as_mut()
        .expect("root host should have computed shell geometry");
    let left = geometry.region_frame(ShellRegionId::Left);
    geometry.region_frames.insert(
        ShellRegionId::Left,
        ShellFrame::new(left.x, left.y, 80.0, left.height),
    );

    host.host_resize_pointer_event(
        None,
        0,
        splitter.x + splitter.width * 0.5,
        splitter.y + splitter.height * 0.5,
    );

    assert_eq!(
        host.active_drawer_resize
            .as_ref()
            .map(|active| active.base_preferred),
        Some(expected_width),
        "resize capture should start from the Workbench drawer shell extent instead of stale legacy geometry"
    );
}

#[test]
fn root_native_focus_loss_cancels_drawer_resize_without_persisting_transient_extent() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_root_resize_focus_cancel");
    harness.activate_workbench_page();
    let native_splitter = harness
        .root_ui
        .get_host_presentation()
        .host_scene_data
        .resize_layer
        .left_splitter_frame;
    assert!(native_splitter.width > 0.0 && native_splitter.height > 0.0);
    let (x, y, committed_width) = {
        let host = harness.host.borrow();
        let committed_width = host
            .shell_geometry
            .as_ref()
            .expect("Workbench should have committed shell geometry")
            .region_frame(ShellRegionId::Left)
            .width;
        (
            native_splitter.x + native_splitter.width * 0.5,
            native_splitter.y + native_splitter.height * 0.5,
            committed_width,
        )
    };
    let baseline = harness.journal_len();

    harness.root_ui.dispatch_native_primary_press_for_test(x, y);
    assert!(
        host_context(&harness.root_ui)
            .get_resize_state()
            .resize_active
    );
    assert!(harness.host.borrow().active_drawer_resize.is_some());
    harness
        .root_ui
        .dispatch_native_pointer_move_for_test(x + 40.0, y);
    assert!(harness
        .host
        .borrow()
        .transient_region_preferred
        .contains_key(&ShellRegionId::Left));
    harness.host.borrow_mut().refresh_ui();
    let transient_width = harness
        .host
        .borrow()
        .shell_geometry
        .as_ref()
        .expect("drawer move should update shell geometry")
        .region_frame(ShellRegionId::Left)
        .width;
    assert!(transient_width > committed_width);

    harness.root_ui.dispatch_native_focus_lost_for_test();
    assert!(
        !host_context(&harness.root_ui)
            .get_resize_state()
            .resize_active
    );
    {
        let host = harness.host.borrow();
        assert!(host.active_drawer_resize.is_none());
        assert!(!host
            .transient_region_preferred
            .contains_key(&ShellRegionId::Left));
    }
    assert!(
        harness.delta_events_since(baseline).is_empty(),
        "Cancel must not commit drawer resize"
    );
    harness.host.borrow_mut().refresh_ui();
    let restored_width = harness
        .host
        .borrow()
        .shell_geometry
        .as_ref()
        .expect("left drawer after canceled refresh")
        .region_frame(ShellRegionId::Left)
        .width;
    assert_eq!(restored_width, committed_width);

    harness
        .root_ui
        .dispatch_native_primary_release_for_test(x + 40.0, y);
    assert!(
        harness.delta_events_since(baseline).is_empty(),
        "late owner Up must not commit resize"
    );
    harness.root_ui.dispatch_native_primary_press_for_test(x, y);
    assert!(
        host_context(&harness.root_ui)
            .get_resize_state()
            .resize_active
    );
    assert!(harness.host.borrow().active_drawer_resize.is_some());
    harness.root_ui.dispatch_native_focus_lost_for_test();
    assert!(harness.delta_events_since(baseline).is_empty());
}

#[test]
fn root_hide_close_cancels_drawer_resize_without_persisting_transient_extent() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_root_resize_close_cancel");
    harness.activate_workbench_page();
    let close_host = Rc::downgrade(&harness.host);
    harness.root_ui.window().on_close_requested(move || {
        close_host.upgrade().map_or(
            crate::ui::retained_host::primitives::CloseRequestResponse::KeepWindowShown,
            |host| host.borrow_mut().native_main_window_close_requested(),
        )
    });
    let native_splitter = harness
        .root_ui
        .get_host_presentation()
        .host_scene_data
        .resize_layer
        .left_splitter_frame;
    assert!(native_splitter.width > 0.0 && native_splitter.height > 0.0);
    let (x, y, committed_width) = {
        let host = harness.host.borrow();
        let committed_width = host
            .shell_geometry
            .as_ref()
            .expect("Workbench should have committed shell geometry")
            .region_frame(ShellRegionId::Left)
            .width;
        (
            native_splitter.x + native_splitter.width * 0.5,
            native_splitter.y + native_splitter.height * 0.5,
            committed_width,
        )
    };
    let baseline = harness.journal_len();

    harness.root_ui.dispatch_native_primary_press_for_test(x, y);
    harness
        .root_ui
        .dispatch_native_pointer_move_for_test(x + 40.0, y);
    assert!(
        host_context(&harness.root_ui)
            .get_resize_state()
            .resize_active
    );
    assert!(harness.host.borrow().active_drawer_resize.is_some());
    assert!(harness
        .host
        .borrow()
        .transient_region_preferred
        .contains_key(&ShellRegionId::Left));
    assert_eq!(
        harness.root_ui.dispatch_native_close_request_for_test(),
        crate::ui::retained_host::primitives::CloseRequestResponse::HideWindow
    );

    assert!(!harness.root_ui.window().is_visible());
    assert!(
        !host_context(&harness.root_ui)
            .get_resize_state()
            .resize_active
    );
    {
        let host = harness.host.borrow();
        assert!(host.active_drawer_resize.is_none());
        assert!(!host
            .transient_region_preferred
            .contains_key(&ShellRegionId::Left));
    }
    assert!(harness.delta_events_since(baseline).is_empty());
    harness.host.borrow_mut().refresh_ui();
    let restored_width = harness
        .host
        .borrow()
        .shell_geometry
        .as_ref()
        .expect("left drawer after canceled close")
        .region_frame(ShellRegionId::Left)
        .width;
    assert_eq!(restored_width, committed_width);

    harness
        .root_ui
        .dispatch_native_primary_release_for_test(x + 40.0, y);
    assert!(harness.delta_events_since(baseline).is_empty());
    harness
        .root_ui
        .show()
        .expect("root host can reopen for a new gesture");
    harness.root_ui.dispatch_native_primary_press_for_test(x, y);
    assert!(
        host_context(&harness.root_ui)
            .get_resize_state()
            .resize_active
    );
    assert!(harness.host.borrow().active_drawer_resize.is_some());
    harness.root_ui.dispatch_native_focus_lost_for_test();
    assert!(harness.delta_events_since(baseline).is_empty());
}

#[test]
fn removed_child_resize_owner_rolls_back_before_recompute_and_preserves_other_window_capture() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_removed_child_resize_owner");
    harness.activate_workbench_page();
    let owner_id = MainPageId::new("window:resize-owner");
    let other_id = MainPageId::new("window:resize-other");
    let owner = harness.detach_view_to_child_window("editor.console#1", owner_id.0.as_str());
    let other = harness.detach_view_to_child_window("editor.hierarchy#1", other_id.0.as_str());
    let (x, y, committed_width) = {
        let host = harness.host.borrow();
        let frames = host.workbench_window_bridge.layout_frames();
        let splitter = frames
            .left_resize_splitter_frame
            .expect("Workbench should expose the left resize splitter");
        let width = host
            .shell_geometry
            .as_ref()
            .expect("Workbench should have committed shell geometry")
            .region_frame(ShellRegionId::Left)
            .width;
        (
            splitter.x + splitter.width * 0.5,
            splitter.y + splitter.height * 0.5,
            width,
        )
    };
    let baseline = harness.journal_len();

    host_context(&owner).set_resize_state(crate::ui::retained_host::HostResizeStateData {
        resize_active: true,
        ..Default::default()
    });
    host_context(&owner).invoke_host_resize_pointer_event(0, x, y);
    assert_eq!(
        harness
            .host
            .borrow()
            .active_drawer_resize
            .as_ref()
            .and_then(|active| active.source_window.as_ref()),
        Some(&owner_id),
        "the child callback should register its own resize source"
    );

    host_context(&other).set_resize_state(crate::ui::retained_host::HostResizeStateData {
        resize_active: true,
        ..Default::default()
    });
    host_context(&other).invoke_host_resize_pointer_event(1, x + 90.0, y);
    host_context(&harness.root_ui).invoke_host_resize_pointer_event(2, x + 90.0, y);
    assert!(
        !harness
            .host
            .borrow()
            .transient_region_preferred
            .contains_key(&ShellRegionId::Left),
        "foreign child and root callbacks must not move or finish the child owner's resize"
    );

    host_context(&owner).invoke_host_resize_pointer_event(1, x + 40.0, y);
    let transient_width = harness
        .host
        .borrow()
        .transient_region_preferred
        .get(&ShellRegionId::Left)
        .copied()
        .expect("owner Move should publish a transient preferred width");
    assert!(transient_width > committed_width);
    harness.host.borrow_mut().refresh_ui();
    let moved_width = harness
        .host
        .borrow()
        .shell_geometry
        .as_ref()
        .expect("owner Move should update Workbench geometry")
        .region_frame(ShellRegionId::Left)
        .width;
    assert!(moved_width > committed_width);

    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_floating_window_close_requested(&owner_id),
        crate::ui::retained_host::primitives::CloseRequestResponse::HideWindow
    );

    {
        let host = harness.host.borrow();
        assert!(host.active_drawer_resize.is_none());
        assert!(!host
            .transient_region_preferred
            .contains_key(&ShellRegionId::Left));
        assert_eq!(
            host.shell_geometry
                .as_ref()
                .expect("same recompute should restore committed geometry")
                .region_frame(ShellRegionId::Left)
                .width,
            committed_width
        );
    }
    assert!(
        !host_context(&owner).get_resize_state().resize_active,
        "the retired owner native capture should be clear"
    );
    assert!(
        host_context(&other).get_resize_state().resize_active,
        "retiring one window must preserve another window's native capture"
    );
    host_context(&owner).invoke_host_resize_pointer_event(2, x + 40.0, y);
    assert!(
        !harness
            .delta_events_since(baseline)
            .iter()
            .any(|event| matches!(
                event,
                EditorEvent::Layout(EventLayoutCommand::SetDrawerRegionExtent { .. })
            )),
        "owner removal and late Up must not commit a resize extent"
    );
}
