use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::WORKBENCH_POPUP_ROW_DANGER_TEXT as POPUP_ROW_DANGER_TEXT;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{
    changed_pixel_count, dropdown_node, dropdown_popup_node, pixel_at, popup_menu_node,
};
use crate::ui::layouts::common::model_rc;

#[test]
fn open_popup_menu_paints_right_aligned_item_icons() {
    let bytes = paint_template_nodes_for_test(180, 180, model_rc(vec![popup_menu_node()]));

    assert!(changed_pixel_count(&bytes, 180, 112, 16, 24, 24) > 0);
    assert_eq!(pixel_at(&bytes, 180, 119, 113), POPUP_ROW_DANGER_TEXT);
    assert!(changed_pixel_count(&bytes, 180, 112, 136, 24, 24) > 0);
}

#[test]
fn selected_dropdown_option_paints_right_check_adornment() {
    let bytes = paint_template_nodes_for_test(150, 120, model_rc(vec![dropdown_node()]));

    assert!(changed_pixel_count(&bytes, 150, 96, 50, 22, 22) > 0);
}

#[test]
fn standalone_dropdown_popup_paints_rows_inside_projected_popup_frame() {
    let bytes = paint_template_nodes_for_test(180, 140, model_rc(vec![dropdown_popup_node()]));

    assert_eq!(pixel_at(&bytes, 180, 20, 20), PALETTE.focus_ring);
}
