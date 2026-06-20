use super::*;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, TemplateNodeFrameData, TemplatePaneNodeData,
};

#[test]
fn dropdown_option_popup_frame_within_opens_above_when_below_overflows() {
    let control = rect(20.0, 120.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 160.0);

    let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
        .expect("popup should have a frame");

    assert_eq!(popup.x, 20.0);
    assert_eq!(popup.y, 32.0);
    assert_eq!(popup.width, 100.0);
    assert_eq!(popup.height, 84.0);
}

#[test]
fn dropdown_option_popup_frame_within_keeps_default_when_above_also_overflows() {
    let control = rect(20.0, 12.0, 100.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 72.0);

    let popup = dropdown_option_popup_frame_within(&control, 3, &bounds)
        .expect("popup should have a frame");

    assert_eq!(popup.y, 44.0);
}

#[test]
fn dropdown_option_popup_frame_within_clamps_right_edge() {
    let control = rect(120.0, 20.0, 80.0, 28.0);
    let bounds = rect(0.0, 0.0, 160.0, 160.0);

    let popup = dropdown_option_popup_frame_within(&control, 2, &bounds)
        .expect("popup should have a frame");

    assert_eq!(popup.x, 80.0);
    assert_eq!(popup.width, 80.0);
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
