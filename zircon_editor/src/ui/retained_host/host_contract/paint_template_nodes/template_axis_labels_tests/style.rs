use crate::ui::retained_host::primitives::Color;

use super::super::scale_link::scale_link_origin;
use super::super::style::{axis_label_color, AXIS_LABEL_COLOR, AXIS_LABEL_SCALE_COLOR};
use super::support::{frame_rect, label_node};

#[test]
fn transform_axis_label_uses_audited_axis_tones() {
    let mut position_axis = label_node("WorkbenchTransformPositionAxisX", "X");
    position_axis.label_color = Color::from_rgb_u8(86, 104, 113);
    assert_eq!(axis_label_color(&position_axis), [86, 104, 113, 255]);
    assert_eq!(
        axis_label_color(&label_node("WorkbenchTransformRotationAxisY", "Y")),
        AXIS_LABEL_COLOR
    );
    assert_eq!(
        axis_label_color(&label_node("WorkbenchTransformScaleAxisZ", "Z")),
        AXIS_LABEL_SCALE_COLOR
    );
}

#[test]
fn scale_link_label_honors_audited_icon_offset() {
    let mut node = label_node("WorkbenchTransformScaleLink", "");
    node.layout_offset_x = -12.0;
    let (start_x, start_y) = scale_link_origin(&node, &frame_rect(&node.frame));

    assert_eq!(start_x, 0.0);
    assert_eq!(start_y, 16.5);
}
