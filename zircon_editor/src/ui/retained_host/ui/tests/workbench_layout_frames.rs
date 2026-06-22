use super::*;

#[test]
fn apply_presentation_uses_workbench_layout_frames_for_document_and_viewport() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let center_frame = UiFrame::new(5.0, 17.0, 400.0, 500.0);
    let document_frame = UiFrame::new(357.0, 57.0, 615.0, 458.0);
    let status_frame = UiFrame::new(11.0, 696.0, 700.0, 24.0);
    let viewport_frame = UiFrame::new(357.0, 114.0, 615.0, 401.0);
    let componentized_workbench_layout_frames =
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
            center_band_frame: Some(center_frame),
            document_region_frame: Some(document_frame),
            status_bar_frame: Some(status_frame),
            viewport_content_frame: Some(viewport_frame),
            ..Default::default()
        };
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation_with_workbench_layout_frames(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.center_band_frame,
        frame_rect_from_ui_frame(center_frame)
    );
    assert_eq!(
        host_layout.document_region_frame,
        frame_rect_from_ui_frame(document_frame)
    );
    assert_eq!(
        host_layout.status_bar_frame,
        frame_rect_from_ui_frame(status_frame)
    );
    assert_eq!(
        host_layout.viewport_content_frame,
        frame_rect_from_ui_frame(viewport_frame)
    );
}

#[test]
fn apply_presentation_leaves_viewport_default_when_workbench_viewport_frame_is_missing() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let mut projection_frames = bridge.root_shell_frames();
    projection_frames.pane_surface_frame = Some(UiFrame::new(369.0, 100.0, 602.0, 420.0));
    let document_frame = UiFrame::new(44.0 + 312.0 + 1.0, 57.0, 1280.0 - 312.0 - 1.0, 639.0);
    let componentized_workbench_layout_frames =
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
            left_region_frame: Some(UiFrame::new(44.0, 57.0, 312.0, 480.0)),
            document_region_frame: Some(document_frame),
            viewport_content_frame: None,
            ..Default::default()
        };

    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        region_frames: [(
            crate::ui::workbench::autolayout::ShellRegionId::Document,
            crate::ui::workbench::autolayout::ShellFrame::new(734.0, 91.0, 222.0, 109.0),
        )]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation_with_workbench_layout_frames(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.document_region_frame,
        frame_rect_from_ui_frame(document_frame)
    );
    assert_eq!(
        host_layout.viewport_content_frame,
        crate::ui::retained_host::FrameRect::default()
    );
}

#[test]
fn apply_presentation_prefers_workbench_layout_frames_for_visible_drawer_region_positions() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let left_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(180.0, 91.0, 312.0, 519.0);
    let right_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(1024.0, 117.0, 256.0, 401.0);
    let bottom_geometry =
        crate::ui::workbench::autolayout::ShellFrame::new(48.0, 712.0, 777.0, 180.0);
    let componentized_workbench_layout_frames =
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
            left_region_frame: Some(UiFrame::new(
                left_geometry.x,
                left_geometry.y,
                left_geometry.width,
                left_geometry.height,
            )),
            right_region_frame: Some(UiFrame::new(
                right_geometry.x,
                right_geometry.y,
                right_geometry.width,
                right_geometry.height,
            )),
            bottom_region_frame: Some(UiFrame::new(
                bottom_geometry.x,
                bottom_geometry.y,
                bottom_geometry.width,
                bottom_geometry.height,
            )),
            ..Default::default()
        };
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                left_geometry,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(493.0, 91.0, 531.0, 440.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                right_geometry,
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                bottom_geometry,
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation_with_workbench_layout_frames(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.left_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .left_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.right_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .right_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.bottom_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .bottom_region_frame
                .unwrap()
        )
    );
}

#[test]
fn apply_presentation_prefers_workbench_layout_frames_for_visible_drawer_region_extents() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let componentized_workbench_layout_frames =
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
            left_region_frame: Some(UiFrame::new(180.0, 91.0, 180.0, 519.0)),
            right_region_frame: Some(UiFrame::new(1024.0, 117.0, 144.0, 401.0)),
            bottom_region_frame: Some(UiFrame::new(48.0, 712.0, 777.0, 120.0)),
            ..Default::default()
        };
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::new(180.0, 91.0, 180.0, 519.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(493.0, 91.0, 531.0, 440.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::new(1024.0, 117.0, 144.0, 401.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::new(48.0, 712.0, 777.0, 120.0),
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation_with_workbench_layout_frames(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.left_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .left_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.right_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .right_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.bottom_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .bottom_region_frame
                .unwrap()
        )
    );
}

#[test]
fn apply_presentation_prefers_workbench_layout_frames_when_legacy_geometry_is_zeroed() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 720,
        ));

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let componentized_workbench_layout_frames =
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
            left_region_frame: Some(UiFrame::new(44.0, 57.0, 312.0, 639.0)),
            document_region_frame: Some(UiFrame::new(357.0, 57.0, 615.0, 458.0)),
            right_region_frame: Some(UiFrame::new(973.0, 57.0, 307.0, 458.0)),
            bottom_region_frame: Some(UiFrame::new(44.0, 516.0, 1236.0, 180.0)),
            viewport_content_frame: Some(UiFrame::new(357.0, 114.0, 615.0, 401.0)),
            ..Default::default()
        };
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::default(),
            ),
        ]
        .into_iter()
        .collect(),
        viewport_content_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            66.0, 120.0, 320.0, 180.0,
        ),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation_with_workbench_layout_frames(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;
    assert_eq!(
        host_layout.left_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .left_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.right_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .right_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.bottom_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .bottom_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.document_region_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .document_region_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.viewport_content_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .viewport_content_frame
                .unwrap()
        )
    );
}

#[test]
fn apply_presentation_resolves_splitters_from_workbench_layout_frames() {
    let (_fixture, chrome, model, ui_asset_panes, animation_panes) = root_shell_fixture();
    let ui =
        crate::ui::retained_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window()
        .set_size(crate::ui::retained_host::primitives::PhysicalSize::new(
            1280, 750,
        ));

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 750.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 750.0),
            &model,
            &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let split_half = metrics.splitter_hit_size * 0.5;
    let componentized_workbench_layout_frames =
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
            right_region_frame: Some(UiFrame::new(620.0, 117.0, 144.0, 320.0)),
            bottom_region_frame: Some(UiFrame::new(0.0, 460.0, 760.0, 120.0)),
            right_resize_splitter_frame: Some(UiFrame::new(
                620.0 - metrics.separator_thickness - split_half,
                117.0,
                metrics.splitter_hit_size,
                320.0,
            )),
            bottom_resize_splitter_frame: Some(UiFrame::new(
                0.0,
                460.0 - metrics.separator_thickness - split_half,
                760.0,
                metrics.splitter_hit_size,
            )),
            ..Default::default()
        };
    let geometry = WorkbenchShellGeometry {
        center_band_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            5.0, 17.0, 400.0, 500.0,
        ),
        status_bar_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            11.0, 520.0, 700.0, 18.0,
        ),
        region_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Left,
                crate::ui::workbench::autolayout::ShellFrame::new(180.0, 91.0, 180.0, 320.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Document,
                crate::ui::workbench::autolayout::ShellFrame::new(493.0, 91.0, 531.0, 320.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::new(620.0, 117.0, 144.0, 320.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::new(0.0, 460.0, 760.0, 120.0),
            ),
        ]
        .into_iter()
        .collect(),
        splitter_frames: [
            (
                crate::ui::workbench::autolayout::ShellRegionId::Right,
                crate::ui::workbench::autolayout::ShellFrame::new(615.0, 17.0, 8.0, 500.0),
            ),
            (
                crate::ui::workbench::autolayout::ShellRegionId::Bottom,
                crate::ui::workbench::autolayout::ShellFrame::new(0.0, 455.0, 760.0, 8.0),
            ),
        ]
        .into_iter()
        .collect(),
        viewport_content_frame: crate::ui::workbench::autolayout::ShellFrame::new(
            66.0, 120.0, 320.0, 180.0,
        ),
        ..WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation_with_workbench_layout_frames(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        Some(&projection_frames),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
    );

    let host_layout = ui.get_host_presentation().host_layout;

    assert_eq!(
        host_layout.right_splitter_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .right_resize_splitter_frame
                .unwrap()
        )
    );
    assert_eq!(
        host_layout.bottom_splitter_frame,
        frame_rect_from_ui_frame(
            componentized_workbench_layout_frames
                .bottom_resize_splitter_frame
                .unwrap()
        )
    );
    assert!(
        host_layout.right_splitter_frame.x >= host_layout.document_region_frame.x,
        "right splitter should stay aligned with the shared right dock instead of stale legacy geometry"
    );
    assert!(
        host_layout.bottom_splitter_frame.y >= host_layout.document_region_frame.y,
        "bottom splitter should stay aligned with the shared bottom dock instead of stale legacy geometry"
    );
}
