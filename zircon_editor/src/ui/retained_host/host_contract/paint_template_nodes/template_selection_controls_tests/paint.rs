use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::PALETTE;
use super::support::{
    changed_pixel_count, checkbox_node, pixel_at, unchecked_checkbox_node, SELECTION_MARK_IDLE_FILL,
};
use crate::ui::layouts::common::model_rc;

#[test]
fn selection_control_paints_checked_checkbox_without_full_row_surface() {
    let bytes = paint_template_nodes_for_test(96, 32, model_rc(vec![checkbox_node()]));

    assert!(changed_pixel_count(&bytes, 96, 8, 7, 18, 18) > 0);
    assert_eq!(pixel_at(&bytes, 96, 19, 12), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 96, 92, 14), [0, 0, 0, 255]);
}

#[test]
fn selection_control_paints_unchecked_mark_surface_without_row_fill() {
    let bytes = paint_template_nodes_for_test(96, 32, model_rc(vec![unchecked_checkbox_node()]));

    assert_eq!(pixel_at(&bytes, 96, 18, 14), SELECTION_MARK_IDLE_FILL);
    assert_eq!(pixel_at(&bytes, 96, 92, 14), [0, 0, 0, 255]);
}
