use super::*;

#[test]
fn builtin_host_window_template_bridge_recomputes_surface_backed_frames_with_shell_size() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let initial = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist")
        .frame;
    assert_eq!(initial, UiFrame::new(44.0, 57.0, 1236.0, 639.0));

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    let recomputed = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist after recompute")
        .frame;
    assert_eq!(recomputed, UiFrame::new(44.0, 57.0, 916.0, 459.0));

    assert_eq!(
        bridge.control_frame("PaneSurfaceRoot"),
        Some(UiFrame::new(44.0, 89.0, 916.0, 427.0))
    );

    let root_frames = bridge.root_shell_frames();
    assert_eq!(
        root_frames.shell_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 540.0))
    );
    assert_eq!(
        root_frames.menu_bar_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 24.0))
    );
    assert_eq!(
        root_frames.activity_rail_frame,
        Some(UiFrame::new(0.0, 57.0, 44.0, 459.0))
    );
    assert_eq!(
        root_frames.host_page_strip_frame,
        Some(UiFrame::new(0.0, 24.0, 960.0, 32.0))
    );
    assert_eq!(
        root_frames.host_body_frame,
        Some(UiFrame::new(0.0, 57.0, 960.0, 459.0))
    );
    assert_eq!(
        root_frames.document_host_frame,
        Some(UiFrame::new(44.0, 57.0, 916.0, 459.0))
    );
    assert_eq!(
        root_frames.document_tabs_frame,
        Some(UiFrame::new(44.0, 57.0, 916.0, 32.0))
    );
    assert_eq!(
        root_frames.pane_surface_frame,
        Some(UiFrame::new(44.0, 89.0, 916.0, 427.0))
    );
    assert_eq!(
        root_frames.status_bar_frame,
        Some(UiFrame::new(0.0, 516.0, 960.0, 24.0))
    );
}

#[test]
fn builtin_host_window_template_bridge_does_not_export_drawer_shell_or_header_frames() {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .unwrap();

    let root_frames = bridge.root_shell_frames();
    assert!(root_frames.host_body_frame.is_some());
    assert!(root_frames.document_host_frame.is_some());
    assert!(bridge.control_frame("LeftDrawerShellRoot").is_none());
    assert!(bridge.control_frame("LeftDrawerHeaderRoot").is_none());
    assert!(bridge.control_frame("RightDrawerShellRoot").is_none());
    assert!(bridge.control_frame("RightDrawerHeaderRoot").is_none());
    assert!(bridge.control_frame("BottomDrawerShellRoot").is_none());
    assert!(bridge.control_frame("BottomDrawerHeaderRoot").is_none());
}

#[test]
fn componentized_workbench_layout_frames_own_drawer_shell_and_header_frames() {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .unwrap();

    let layout_frames = bridge.layout_frames();
    assert_eq!(
        layout_frames.left_drawer_shell_frame,
        bridge.control_frame("LeftDrawerShellRoot")
    );
    assert_eq!(
        layout_frames.left_drawer_header_frame,
        bridge.control_frame("LeftDrawerHeaderRoot")
    );
    assert_eq!(
        layout_frames.right_drawer_shell_frame,
        bridge.control_frame("RightDrawerShellRoot")
    );
    assert_eq!(
        layout_frames.right_drawer_header_frame,
        bridge.control_frame("RightDrawerHeaderRoot")
    );
    assert_eq!(
        layout_frames.bottom_drawer_shell_frame,
        bridge.control_frame("BottomDrawerShellRoot")
    );
    assert_eq!(
        layout_frames.bottom_drawer_header_frame,
        bridge.control_frame("BottomDrawerHeaderRoot")
    );
}

#[test]
fn componentized_workbench_layout_frames_own_drawer_content_frames() {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .unwrap();

    let layout_frames = bridge.layout_frames();
    assert_eq!(
        layout_frames.left_drawer_content_frame,
        bridge.control_frame("LeftDrawerContentRoot")
    );
    assert_eq!(
        layout_frames.right_drawer_content_frame,
        bridge.control_frame("RightDrawerContentRoot")
    );
    assert_eq!(
        layout_frames.bottom_drawer_content_frame,
        bridge.control_frame("BottomDrawerContentRoot")
    );
}

#[test]
fn componentized_narrow_workbench_keeps_a_token_sized_bottom_drawer_reopen_strip() {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let shell_size = UiSize::new(640.0, 420.0);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size).unwrap();

    bridge
        .recompute_layout_with_workbench_model(shell_size, &model, &metrics)
        .unwrap();

    let layout_frames = bridge.layout_frames();
    let bottom_shell = layout_frames
        .bottom_drawer_shell_frame
        .expect("narrow workbench should retain the bottom drawer reopen strip");
    let bottom_header = layout_frames
        .bottom_drawer_header_frame
        .expect("narrow workbench should retain the bottom drawer header");

    assert_eq!(bottom_shell.height, metrics.panel_header_height);
    assert_eq!(bottom_header.height, metrics.panel_header_height);
    assert_eq!(layout_frames.bottom_drawer_content_frame, None);
}
