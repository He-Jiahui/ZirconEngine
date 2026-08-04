use super::metrics::TEMPLATE_POPUP_ANCHOR_GAP;
use super::*;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::popup_anchor_metrics::{
    clamp_popup_x_to_bounds, SLATE_POPUP_ANCHOR_METRICS,
};

#[test]
fn dropdown_option_popup_frame_within_opens_above_when_below_overflows() {
    let control = rect(20.0, 120.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 160.0);

    let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
        .expect("popup should have a frame");

    assert_eq!(popup.x, 20.0);
    assert_eq!(popup.y, 33.0);
    assert_eq!(popup.width, 100.0);
    assert_eq!(popup.height, 84.0);
}

#[test]
fn dropdown_option_popup_frame_within_keeps_default_when_above_also_overflows() {
    let control = rect(20.0, 12.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 72.0);

    let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
        .expect("popup should have a frame");

    assert_eq!(popup.y, 43.0);
}

#[test]
fn dropdown_option_popup_frame_within_clamps_height_to_the_larger_available_side() {
    let control = rect(20.0, 12.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 72.0);

    let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
        .expect("popup should fit the available side");

    assert_eq!(popup.y, 43.0);
    assert_eq!(popup.height, 29.0);
    assert!(popup.y >= bounds.y);
    assert!(popup.y + popup.height <= bounds.y + bounds.height);
}

#[test]
fn dropdown_option_popup_frame_within_clamps_an_offscreen_anchor_to_bounds() {
    let control = rect(20.0, -32.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 72.0);

    let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
        .expect("popup should remain available while its anchor is partially offscreen");

    assert_eq!(popup.y, 0.0);
    assert_eq!(popup.height, 72.0);
    assert!(popup.y >= bounds.y);
    assert!(popup.y + popup.height <= bounds.y + bounds.height);
}

#[test]
fn dropdown_option_row_frame_within_rejects_rows_beyond_a_clamped_popup() {
    let control = rect(20.0, 12.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 72.0);

    assert_eq!(
        dropdown_option_row_frame_within(&control, 3, 0, &bounds),
        Some(rect(20.0, 43.0, 100.0, 28.0))
    );
    assert_eq!(
        dropdown_option_row_frame_within(&control, 3, 1, &bounds),
        None
    );
}

#[test]
fn dropdown_option_popup_frame_within_clamps_right_edge() {
    let control = rect(120.0, 20.0, 80.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 160.0);

    let popup = dropdown_option_popup_frame_within(&control, 2, &bounds)
        .expect("popup should have a frame");

    assert_eq!(popup.x, 72.0);
    assert_eq!(popup.width, 80.0);
}

#[test]
fn dropdown_option_popup_frame_within_uses_shared_anchor_margin_tokens() {
    assert_eq!(
        TEMPLATE_POPUP_ANCHOR_GAP,
        SLATE_POPUP_ANCHOR_METRICS.anchor_gap
    );
    assert_eq!(
        dropdown_option_popup_frame_within(
            &rect(2.0, 20.0, 80.0, 28.0),
            2,
            &rect(0.0, 0.0, 160.0, 160.0),
        )
        .expect("popup should have a frame")
        .x,
        clamp_popup_x_to_bounds(2.0, 0.0, 160.0, 80.0)
    );
}

#[test]
fn template_option_popup_frame_within_uses_projected_dropdown_popup_frame() {
    let node = TemplatePaneNodeData {
        role: "DropdownPopup".into(),
        component_role: "dropdown-popup".into(),
        frame: TemplateNodeFrameData {
            x: 100.0,
            y: 60.0,
            width: 120.0,
            height: 96.0,
        },
        ..TemplatePaneNodeData::default()
    };
    let popup = template_option_popup_frame_within(
        &node,
        &rect(100.0, 60.0, 120.0, 96.0),
        4,
        &rect(0.0, 0.0, 320.0, 240.0),
    )
    .expect("DropdownPopup should use its projected popup frame");
    let row = template_option_row_frame_within(&node, &popup, 4, 2, &rect(0.0, 0.0, 320.0, 240.0))
        .expect("DropdownPopup row should be inside the projected popup frame");

    assert_eq!(popup, rect(100.0, 60.0, 120.0, 96.0));
    assert_eq!(row, rect(100.0, 108.0, 120.0, 24.0));
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
