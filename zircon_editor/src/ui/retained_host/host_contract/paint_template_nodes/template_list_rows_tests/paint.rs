use crate::ui::layouts::common::model_rc;

use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, list_node, pixel_at};

#[test]
fn selected_list_row_paints_surface_and_right_check() {
    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![list_node(true, false)]));

    assert_ne!(pixel_at(&bytes, 160, 12, 18), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}

#[test]
fn disabled_list_row_keeps_background_empty_and_draws_disabled_adornment() {
    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![list_node(false, true)]));

    assert_eq!(pixel_at(&bytes, 160, 12, 18), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}
