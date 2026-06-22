use crate::ui::layouts::common::model_rc;

use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_axis_value_field_style::{
    AXIS_FIELD_BACKGROUND, AXIS_FIELD_BORDER, AXIS_FIELD_DISABLED_BACKGROUND,
    AXIS_FIELD_DISABLED_BORDER, AXIS_FIELD_HOVER_BACKGROUND,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{axis_node, changed_pixel_count, pixel_at};

#[test]
fn axis_value_field_paints_compact_field_and_value() {
    let bytes = paint_template_nodes_for_test(
        96,
        48,
        model_rc(vec![axis_node("WorkbenchTransformPositionX", "128.4")]),
    );

    assert_eq!(pixel_at(&bytes, 96, 22, 8), AXIS_FIELD_BORDER);
    assert_eq!(pixel_at(&bytes, 96, 60, 18), AXIS_FIELD_BACKGROUND);
    assert!(changed_pixel_count(&bytes, 96, 16, 12, 44, 18) > 0);
}

#[test]
fn focused_axis_value_field_uses_focus_border() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.focused = true;

    let bytes = paint_template_nodes_for_test(96, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 96, 22, 8), PALETTE.focus_ring);
    assert_eq!(pixel_at(&bytes, 96, 18, 18), AXIS_FIELD_HOVER_BACKGROUND);
}

#[test]
fn disabled_axis_value_field_uses_muted_surface() {
    let mut node = axis_node("WorkbenchTransformScaleZ", "1.00");
    node.disabled = true;

    let bytes = paint_template_nodes_for_test(96, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 96, 22, 8), AXIS_FIELD_DISABLED_BORDER);
    assert_eq!(pixel_at(&bytes, 96, 60, 18), AXIS_FIELD_DISABLED_BACKGROUND,);
}
