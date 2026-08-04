use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::WORKBENCH_POPUP_ROW_DANGER_TEXT as POPUP_ROW_DANGER_TEXT;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{
    changed_pixel_count, dropdown_node, dropdown_popup_node, matching_pixel_count, pixel_at,
    popup_menu_node,
};
use crate::ui::layouts::common::model_rc;

#[test]
fn open_popup_menu_paints_right_aligned_item_icons() {
    let bytes = paint_template_nodes_for_test(180, 180, model_rc(vec![popup_menu_node()]));

    assert!(changed_pixel_count(&bytes, 180, 112, 16, 24, 24) > 0);
    assert!(matching_pixel_count(&bytes, 180, POPUP_ROW_DANGER_TEXT, 28, 100, 72, 30) > 0);
    assert!(changed_pixel_count(&bytes, 180, 112, 136, 24, 24) > 0);
}

#[test]
fn selected_dropdown_option_paints_right_check_adornment() {
    let bytes = paint_template_nodes_for_test(150, 120, model_rc(vec![dropdown_node()]));

    assert!(matching_pixel_count(&bytes, 150, PALETTE.accent, 12, 44, 112, 28) > 64);
    assert!(matching_pixel_count(&bytes, 150, PALETTE.surface_pressed, 14, 46, 108, 24) > 1_500);
    assert_eq!(
        matching_pixel_count(&bytes, 150, PALETTE.surface_selected, 14, 46, 108, 24),
        0
    );
    assert_eq!(
        matching_pixel_count(&bytes, 150, PALETTE.accent, 13, 52, 1, 12),
        0
    );
    assert!(changed_pixel_count(&bytes, 150, 96, 50, 22, 22) > 0);
}

#[test]
fn standalone_dropdown_popup_paints_selected_row_outline_inside_projected_popup_frame() {
    let bytes = paint_template_nodes_for_test(180, 140, model_rc(vec![dropdown_popup_node()]));

    assert_eq!(pixel_at(&bytes, 180, 20, 20), PALETTE.accent);
    assert!(matching_pixel_count(&bytes, 180, PALETTE.accent, 20, 16, 120, 24) > 72);
    assert!(matching_pixel_count(&bytes, 180, PALETTE.surface_pressed, 22, 18, 116, 20) > 1_800);
    assert_eq!(
        matching_pixel_count(&bytes, 180, PALETTE.surface_selected, 22, 18, 116, 20),
        0
    );
    assert_eq!(
        matching_pixel_count(&bytes, 180, PALETTE.accent, 21, 24, 1, 12),
        0
    );
}
