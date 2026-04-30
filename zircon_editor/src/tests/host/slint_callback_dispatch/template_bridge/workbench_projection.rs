use super::super::support::*;
use crate::ui::workbench::autolayout::{compact_bottom_height_limit, WorkbenchChromeMetrics};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn builtin_host_window_template_bridge_recomputes_surface_backed_frames_with_shell_size() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let initial = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist")
        .frame;
    assert_eq!(initial, UiFrame::new(56.0, 59.0, 1224.0, 637.0));

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    let recomputed = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist after recompute")
        .frame;
    assert_eq!(recomputed, UiFrame::new(56.0, 59.0, 904.0, 457.0));

    assert_eq!(
        bridge.control_frame("PaneSurfaceRoot"),
        Some(UiFrame::new(56.0, 91.0, 904.0, 425.0))
    );

    let root_frames = bridge.root_shell_frames();
    assert_eq!(
        root_frames.shell_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 540.0))
    );
    assert_eq!(
        root_frames.menu_bar_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 26.0))
    );
    assert_eq!(
        root_frames.activity_rail_frame,
        Some(UiFrame::new(0.0, 59.0, 56.0, 457.0))
    );
    assert_eq!(
        root_frames.host_page_strip_frame,
        Some(UiFrame::new(0.0, 26.0, 960.0, 32.0))
    );
    assert_eq!(
        root_frames.host_body_frame,
        Some(UiFrame::new(0.0, 59.0, 960.0, 457.0))
    );
    assert_eq!(
        root_frames.document_host_frame,
        Some(UiFrame::new(56.0, 59.0, 904.0, 457.0))
    );
    assert_eq!(
        root_frames.document_tabs_frame,
        Some(UiFrame::new(56.0, 59.0, 904.0, 32.0))
    );
    assert_eq!(
        root_frames.pane_surface_frame,
        Some(UiFrame::new(56.0, 91.0, 904.0, 425.0))
    );
    assert_eq!(
        root_frames.status_bar_frame,
        Some(UiFrame::new(0.0, 516.0, 960.0, 24.0))
    );
}

#[test]
fn builtin_host_window_template_bridge_exports_visible_drawer_shell_and_header_frames_from_workbench_model(
) {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .unwrap();

    let root_frames = bridge.root_shell_frames();
    let body_frame = root_frames
        .host_body_frame
        .expect("workbench body projection frame should exist");
    let metrics = WorkbenchChromeMetrics::default();
    let requested_bottom_height = 164.0_f32;
    let expected_bottom_height =
        compact_bottom_height_limit((body_frame.height - metrics.separator_thickness).max(0.0))
            .map(|limit| requested_bottom_height.min(limit))
            .unwrap_or(requested_bottom_height);
    let expected_bottom_height = round_to_centipixel(expected_bottom_height);
    let expected_center_height = round_to_centipixel(
        body_frame.height - expected_bottom_height - metrics.separator_thickness,
    );
    let expected_bottom_y =
        round_to_centipixel(body_frame.y + body_frame.height - expected_bottom_height);
    let expected_bottom_content_height = round_to_centipixel(
        (expected_bottom_height - metrics.panel_header_height - metrics.separator_thickness)
            .max(0.0),
    );
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
            expected_bottom_y,
            body_frame.width,
            expected_bottom_height,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_header_frame,
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y,
            body_frame.width,
            25.0,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y + metrics.panel_header_height + metrics.separator_thickness,
            body_frame.width,
            expected_bottom_content_height,
        ))
    );
}

fn round_to_centipixel(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}
