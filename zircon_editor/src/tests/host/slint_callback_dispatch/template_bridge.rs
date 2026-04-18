use super::support::*;

#[test]
fn builtin_workbench_drawer_source_document_can_be_loaded_as_shared_surface() {
    let _guard = env_lock().lock().unwrap();

    let mut runtime = crate::host::template_runtime::EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    let mut surface = runtime
        .build_shared_surface("workbench.drawer_source")
        .expect("drawer source document should be registered as a builtin shared surface");
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    assert_eq!(
        surface_control_frame(&surface, "WorkbenchDrawerTopBarRoot"),
        Some(UiFrame::new(0.0, 0.0, 1280.0, 40.0))
    );
    assert_eq!(
        surface_control_frame(&surface, "WorkbenchDrawerStatusBarRoot"),
        Some(UiFrame::new(0.0, 696.0, 1280.0, 24.0))
    );
}

#[test]
fn builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let initial = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist")
        .frame;
    assert_eq!(initial, UiFrame::new(56.0, 40.0, 1224.0, 656.0));

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    let recomputed = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist after recompute")
        .frame;
    assert_eq!(recomputed, UiFrame::new(56.0, 40.0, 904.0, 476.0));

    assert_eq!(
        bridge.control_frame("PaneSurfaceRoot"),
        Some(UiFrame::new(56.0, 72.0, 904.0, 444.0))
    );

    let root_frames = bridge.root_shell_frames();
    assert_eq!(
        root_frames.shell_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 540.0))
    );
    assert_eq!(
        root_frames.menu_bar_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 40.0))
    );
    assert_eq!(
        root_frames.activity_rail_frame,
        Some(UiFrame::new(0.0, 40.0, 56.0, 476.0))
    );
    assert_eq!(
        root_frames.host_page_strip_frame,
        Some(UiFrame::new(0.0, 26.0, 960.0, 24.0))
    );
    assert_eq!(
        root_frames.workbench_body_frame,
        Some(UiFrame::new(0.0, 40.0, 960.0, 476.0))
    );
    assert_eq!(
        root_frames.document_host_frame,
        Some(UiFrame::new(56.0, 40.0, 904.0, 476.0))
    );
    assert_eq!(
        root_frames.document_tabs_frame,
        Some(UiFrame::new(56.0, 40.0, 904.0, 32.0))
    );
    assert_eq!(
        root_frames.pane_surface_frame,
        Some(UiFrame::new(56.0, 72.0, 904.0, 444.0))
    );
    assert_eq!(
        root_frames.status_bar_frame,
        Some(UiFrame::new(0.0, 516.0, 960.0, 24.0))
    );
}

#[test]
fn builtin_workbench_template_bridge_exports_visible_drawer_shell_and_header_frames_from_workbench_model(
) {
    let _guard = env_lock().lock().unwrap();

    let fixture = crate::default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = crate::WorkbenchViewModel::build(&chrome);
    let mut bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::WorkbenchChromeMetrics::default(),
        )
        .unwrap();

    let root_frames = bridge.root_shell_frames();
    let body_frame = root_frames
        .workbench_body_frame
        .expect("workbench body projection frame should exist");
    let expected_center_height = body_frame.height - 164.0 - 1.0;
    assert_eq!(
        root_frames.left_drawer_shell_frame,
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y,
            312.0,
            expected_center_height
        ))
    );
    assert_eq!(
        root_frames.left_drawer_header_frame,
        Some(UiFrame::new(body_frame.x + 35.0, body_frame.y, 277.0, 25.0))
    );
    assert_eq!(
        root_frames.left_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x + 35.0,
            body_frame.y + 26.0,
            277.0,
            expected_center_height - 26.0,
        ))
    );
    assert_eq!(
        root_frames.right_drawer_shell_frame,
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y,
            308.0,
            expected_center_height,
        ))
    );
    assert_eq!(
        root_frames.right_drawer_header_frame,
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y,
            273.0,
            25.0,
        ))
    );
    assert_eq!(
        root_frames.right_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y + 26.0,
            273.0,
            expected_center_height - 26.0,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_shell_frame,
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - 164.0,
            body_frame.width,
            164.0,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_header_frame,
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - 164.0,
            body_frame.width,
            25.0,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - 138.0,
            body_frame.width,
            138.0,
        ))
    );
}

#[test]
fn builtin_workbench_drawer_source_template_bridge_exports_visible_drawer_frames_from_workbench_model(
) {
    let _guard = env_lock().lock().unwrap();

    let fixture = crate::default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = crate::WorkbenchViewModel::build(&chrome);
    let mut bridge =
        BuiltinWorkbenchDrawerSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::WorkbenchChromeMetrics::default(),
        )
        .unwrap();

    let frames = bridge.source_frames();
    let center_height = 720.0 - 40.0 - 24.0 - 1.0 - 164.0;
    assert_eq!(
        frames.left_drawer_shell_frame,
        Some(UiFrame::new(0.0, 40.0, 312.0, center_height))
    );
    assert_eq!(
        frames.left_drawer_header_frame,
        Some(UiFrame::new(35.0, 40.0, 277.0, 25.0))
    );
    assert_eq!(
        frames.left_drawer_content_frame,
        Some(UiFrame::new(35.0, 66.0, 277.0, center_height - 26.0))
    );
    assert_eq!(
        frames.right_drawer_shell_frame,
        Some(UiFrame::new(972.0, 40.0, 308.0, center_height))
    );
    assert_eq!(
        frames.right_drawer_header_frame,
        Some(UiFrame::new(972.0, 40.0, 273.0, 25.0))
    );
    assert_eq!(
        frames.right_drawer_content_frame,
        Some(UiFrame::new(972.0, 66.0, 273.0, center_height - 26.0))
    );
    assert_eq!(
        frames.bottom_drawer_shell_frame,
        Some(UiFrame::new(0.0, 532.0, 1280.0, 164.0))
    );
    assert_eq!(
        frames.bottom_drawer_header_frame,
        Some(UiFrame::new(0.0, 532.0, 1280.0, 25.0))
    );
    assert_eq!(
        frames.bottom_drawer_content_frame,
        Some(UiFrame::new(0.0, 558.0, 1280.0, 138.0))
    );
}

#[test]
fn builtin_floating_window_source_template_bridge_recomputes_surface_backed_frames_with_shell_size()
{
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinFloatingWindowSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    assert_eq!(
        bridge.source_frames().center_band_frame,
        Some(UiFrame::new(0.0, 40.0, 1280.0, 656.0))
    );
    assert_eq!(
        bridge.source_frames().document_frame,
        Some(UiFrame::new(56.0, 40.0, 1224.0, 656.0))
    );

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    assert_eq!(
        bridge.source_frames().center_band_frame,
        Some(UiFrame::new(0.0, 40.0, 960.0, 476.0))
    );
    assert_eq!(
        bridge.source_frames().document_frame,
        Some(UiFrame::new(56.0, 40.0, 904.0, 476.0))
    );
}

fn surface_control_frame(surface: &zircon_ui::UiSurface, control_id: &str) -> Option<UiFrame> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.layout_cache.frame)
    })
}
