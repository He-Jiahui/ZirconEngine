use crate::ui::layouts::common::model_rc;

use super::super::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, popup_menu_node};

#[test]
fn template_nodes_paint_open_popup_menu_rows_inside_menu_frame() {
    let bytes = paint_template_nodes_for_test(160, 128, model_rc(vec![popup_menu_node()]));

    assert!(changed_pixel_count(&bytes, 160, 16, 16, 128, 96) > 0);
}
