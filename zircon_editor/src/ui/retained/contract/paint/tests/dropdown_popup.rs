use crate::ui::layouts::common::model_rc;

use super::super::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, dropdown_near_bottom_node, dropdown_node};

#[test]
fn template_nodes_paint_open_dropdown_option_rows_below_control() {
    let bytes = paint_template_nodes_for_test(128, 128, model_rc(vec![dropdown_node()]));

    assert!(changed_pixel_count(&bytes, 128, 12, 48, 112, 66) > 0);
}

#[test]
fn template_nodes_anchor_workbench_dropdown_popup_to_declared_layout_offset() {
    let mut node = dropdown_node();
    node.control_id = "WorkbenchInputDropdown".into();
    node.layout_offset_x = 10.0;
    node.layout_offset_y = 6.0;
    let bytes = paint_template_nodes_for_test(160, 160, model_rc(vec![node]));

    assert!(changed_pixel_count(&bytes, 160, 22, 54, 112, 66) > 0);
    assert_eq!(changed_pixel_count(&bytes, 160, 12, 44, 8, 84), 0);
}

#[test]
fn template_nodes_paint_open_dropdown_option_rows_above_control_when_below_clipped() {
    let bytes =
        paint_template_nodes_for_test(160, 160, model_rc(vec![dropdown_near_bottom_node()]));

    assert!(changed_pixel_count(&bytes, 160, 20, 32, 100, 84) > 0);
    assert_eq!(changed_pixel_count(&bytes, 160, 20, 152, 100, 8), 0);
}
