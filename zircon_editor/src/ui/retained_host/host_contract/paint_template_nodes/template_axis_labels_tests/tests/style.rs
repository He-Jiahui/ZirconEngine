use crate::ui::retained_host::primitives::Color;

use super::super::palette::axis_label_palette;
use super::super::scale_link::scale_link_origin;
use super::super::style::{axis_label_color, scale_link_color};
use super::support::{frame_rect, label_node};

#[test]
fn transform_axis_label_uses_declared_or_projected_axis_tones() {
    let palette = axis_label_palette();
    let mut position_axis = label_node("WorkbenchTransformPositionAxisX", "X");
    position_axis.label_color = Color::from_rgb_u8(86, 104, 113);
    assert_eq!(axis_label_color(&position_axis), [86, 104, 113, 255]);
    assert_eq!(
        axis_label_color(&label_node("WorkbenchTransformRotationAxisY", "Y")),
        palette.axis
    );
    assert_eq!(
        axis_label_color(&label_node("WorkbenchTransformScaleAxisZ", "Z")),
        palette.scale_axis
    );

    let mut disabled_axis = label_node("WorkbenchTransformRotationAxisX", "X");
    disabled_axis.disabled = true;
    disabled_axis.label_color = Color::from_rgb_u8(255, 255, 255);
    assert_eq!(
        axis_label_color(&disabled_axis),
        palette.disabled_axis,
        "disabled transform axis should keep disabled owner tone above declared color"
    );
}

#[test]
fn scale_link_label_uses_projected_palette_tones() {
    let palette = axis_label_palette();
    assert_eq!(
        scale_link_color(&label_node("WorkbenchTransformScaleLink", "")),
        palette.scale_link
    );

    let mut disabled_link = label_node("WorkbenchTransformScaleLink", "");
    disabled_link.disabled = true;
    assert_eq!(
        scale_link_color(&disabled_link),
        palette.disabled_scale_link
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
