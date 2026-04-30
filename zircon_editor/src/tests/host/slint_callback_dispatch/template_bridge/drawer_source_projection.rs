use super::super::support::*;
use crate::ui::workbench::autolayout::{compact_bottom_height_limit, WorkbenchChromeMetrics};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn builtin_workbench_drawer_source_template_bridge_exports_visible_drawer_frames_from_workbench_model(
) {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        BuiltinHostDrawerSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .unwrap();

    let frames = bridge.source_frames();
    let requested_bottom_height = 164.0_f32;
    let bottom_available_height = (720.0
        - metrics.top_bar_height
        - metrics.separator_thickness
        - metrics.host_bar_height
        - metrics.separator_thickness
        - metrics.status_bar_height
        - metrics.separator_thickness)
        .max(0.0);
    let bottom_height = compact_bottom_height_limit(bottom_available_height)
        .map(|limit| requested_bottom_height.min(limit))
        .unwrap_or(requested_bottom_height);
    let bottom_height = round_to_centipixel(bottom_height);
    let bottom_y = round_to_centipixel(720.0 - 24.0 - bottom_height);
    let center_height = round_to_centipixel(bottom_y - 59.0 - 1.0);
    let bottom_content_height = round_to_centipixel((bottom_height - 26.0).max(0.0));
    assert_eq!(
        frames.left_drawer_shell_frame,
        Some(UiFrame::new(0.0, 59.0, 312.0, center_height))
    );
    assert_eq!(
        frames.left_drawer_header_frame,
        Some(UiFrame::new(35.0, 59.0, 277.0, 25.0))
    );
    assert_eq!(
        frames.left_drawer_content_frame,
        Some(UiFrame::new(35.0, 85.0, 277.0, center_height - 26.0))
    );
    assert_eq!(
        frames.right_drawer_shell_frame,
        Some(UiFrame::new(972.0, 59.0, 308.0, center_height))
    );
    assert_eq!(
        frames.right_drawer_header_frame,
        Some(UiFrame::new(972.0, 59.0, 273.0, 25.0))
    );
    assert_eq!(
        frames.right_drawer_content_frame,
        Some(UiFrame::new(972.0, 85.0, 273.0, center_height - 26.0))
    );
    assert_eq!(
        frames.bottom_drawer_shell_frame,
        Some(UiFrame::new(0.0, bottom_y, 1280.0, bottom_height))
    );
    assert_eq!(
        frames.bottom_drawer_header_frame,
        Some(UiFrame::new(0.0, bottom_y, 1280.0, 25.0))
    );
    assert_eq!(
        frames.bottom_drawer_content_frame,
        Some(UiFrame::new(
            0.0,
            bottom_y + 26.0,
            1280.0,
            bottom_content_height
        ))
    );
}

fn round_to_centipixel(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}
