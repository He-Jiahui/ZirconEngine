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
fn root_document_tab_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry(
) {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_document_tab_projection_width");
    harness.activate_workbench_page();
    let expected_instance = ViewInstanceId::new("editor.scene#1");
    let (tab_index, tab_x, point_x, point_y, tab_width) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(&chrome);
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
            let floating_window_projection_bundle =
                crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle::default();
            host.sync_document_tab_pointer_layout(&model, &floating_window_projection_bundle);
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
    let (tab_x, point_x, point_y, tab_width) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(&chrome);
        let shared_shell_frame = host
            .template_bridge
            .control_frame("UiHostWindowRoot")
            .expect("workbench shell root should map to a projected control frame");
        assert!(
            shared_shell_frame.width > 1000.0,
            "shared shell projection width should outrank host-page metric estimates"
        );
        host.sync_host_page_pointer_layout(&model);

        let tab_x = 8.0;
        let tab_width = 132.0;
        (tab_x, 12.0, 12.0, tab_width)
    };
    let baseline = harness.journal_len();

    host_context(&harness.root_ui)
        .invoke_host_page_pointer_clicked(0, tab_x, tab_width, point_x, point_y);

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
        let model = WorkbenchViewModel::build(&chrome);
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
