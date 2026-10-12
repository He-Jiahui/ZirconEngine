use super::support::*;

#[test]
fn root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_viewport_size_alignment");
    let viewport_frame = harness
        .root_ui
        .get_host_presentation()
        .host_layout
        .viewport_content_frame;
    let host = harness.host.borrow();

    assert_eq!(
        host.viewport_size,
        UVec2::new(
            viewport_frame.width.max(0.0).round() as u32,
            viewport_frame.height.max(0.0).round() as u32,
        )
    );
}

#[test]
fn root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_visible() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_root_viewport_size_visible_drawers");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    let viewport_frame = harness
        .root_ui
        .get_host_presentation()
        .host_layout
        .viewport_content_frame;
    let host = harness.host.borrow();

    assert_eq!(
        host.viewport_size,
        UVec2::new(
            viewport_frame.width.max(0.0).round() as u32,
            viewport_frame.height.max(0.0).round() as u32,
        )
    );
}

#[test]
fn native_frame_request_recomputes_dirty_layout_before_presentation() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_native_frame_recompute");
    harness
        .root_ui
        .window()
        .set_size(PhysicalSize::new(960, 540));

    harness.root_ui.request_host_frame_for_test();

    let presentation = harness.root_ui.get_host_presentation();
    assert_eq!(presentation.host_layout.status_bar_frame.width, 960.0);
    assert_eq!(presentation.host_layout.status_bar_frame.y, 516.0);
}

#[test]
fn root_host_recomputes_builtin_template_bridge_with_visible_drawer_shell_and_header_frames() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_root_visible_drawer_template_frames");
    harness.activate_workbench_page();

    let host = harness.host.borrow();
    let body_frame = host
        .template_bridge
        .control_frame("WorkbenchBody")
        .expect("workbench body control frame should exist");
    let metrics = crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default();
    let requested_bottom_height = 164.0_f32;
    let expected_bottom_height = crate::ui::workbench::autolayout::compact_bottom_height_limit(
        (body_frame.height - metrics.separator_thickness).max(0.0),
    )
    .map(|limit| requested_bottom_height.min(limit))
    .unwrap_or(requested_bottom_height);
    let expected_bottom_height = round_to_layout_pixel(expected_bottom_height);
    let expected_center_height = round_to_layout_pixel(
        body_frame.height - expected_bottom_height - metrics.separator_thickness,
    );
    let expected_bottom_y =
        round_to_layout_pixel(body_frame.y + body_frame.height - expected_bottom_height);
    let expected_bottom_content_height = round_to_layout_pixel(
        (expected_bottom_height - metrics.panel_header_height - metrics.separator_thickness)
            .max(0.0),
    );
    let expected_left_drawer_width = 260.0;
    let expected_left_panel_width =
        expected_left_drawer_width - metrics.rail_width - metrics.separator_thickness;
    let expected_right_drawer_width = 260.0;
    let expected_right_panel_width =
        expected_right_drawer_width - metrics.rail_width - metrics.separator_thickness;
    assert_eq!(
        host.template_bridge.control_frame("LeftDrawerShellRoot"),
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y,
            expected_left_drawer_width,
            expected_center_height
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("LeftDrawerHeaderRoot"),
        Some(UiFrame::new(
            body_frame.x + metrics.rail_width + metrics.separator_thickness,
            body_frame.y,
            expected_left_panel_width,
            25.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("LeftDrawerContentRoot"),
        Some(UiFrame::new(
            body_frame.x + metrics.rail_width + metrics.separator_thickness,
            body_frame.y + 25.0,
            expected_left_panel_width,
            expected_center_height - 25.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("RightDrawerShellRoot"),
        Some(UiFrame::new(
            body_frame.x + body_frame.width - expected_right_drawer_width,
            body_frame.y,
            expected_right_drawer_width,
            expected_center_height,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("RightDrawerHeaderRoot"),
        Some(UiFrame::new(
            body_frame.x + body_frame.width - expected_right_drawer_width,
            body_frame.y,
            expected_right_panel_width,
            25.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("RightDrawerContentRoot"),
        Some(UiFrame::new(
            body_frame.x + body_frame.width - expected_right_drawer_width,
            body_frame.y + 25.0,
            expected_right_panel_width,
            expected_center_height - 25.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("BottomDrawerShellRoot"),
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y,
            body_frame.width,
            expected_bottom_height,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("BottomDrawerHeaderRoot"),
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y,
            body_frame.width,
            25.0,
        ))
    );
    assert_eq!(
        host.template_bridge
            .control_frame("BottomDrawerContentRoot"),
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y + metrics.panel_header_height + metrics.separator_thickness,
            body_frame.width,
            expected_bottom_content_height,
        ))
    );
}

fn round_to_layout_pixel(value: f32) -> f32 {
    value.round()
}
