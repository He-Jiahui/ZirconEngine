use super::super::super::style_selector::{
    WORKBENCH_TABLE_HEADER_BG as TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT as TABLE_HEADER_TEXT,
    WORKBENCH_TABLE_SELECTED_BG as TABLE_SELECTED_BG, WORKBENCH_TABLE_SEPARATOR as TABLE_SEPARATOR,
    WORKBENCH_TABLE_TAIL_BG as TABLE_TAIL_BG,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::style::table_cell_color;
use super::support::{different_pixel_count, pixel_at, table_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_table_row_paints_selected_surface_and_action_glyph() {
    let bytes = paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableSelected", true)]),
    );

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_SELECTED_BG);
    assert!(different_pixel_count(&bytes, 240, 220, 15, 14, 14, TABLE_SELECTED_BG) > 0);
    assert!(different_pixel_count(&bytes, 240, 14, 11, 90, 14, TABLE_SELECTED_BG) > 0);
}

#[test]
fn workbench_table_header_paints_muted_surface_separator_and_gear() {
    let node = table_node("WorkbenchTableHeader", false);
    assert_eq!(table_cell_color(&node, 0), TABLE_HEADER_TEXT);
    let bytes = paint_template_nodes_for_test(240, 44, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_HEADER_BG);
    assert_eq!(pixel_at(&bytes, 240, 8, 31), TABLE_SEPARATOR);
    assert!(different_pixel_count(&bytes, 240, 220, 15, 14, 14, TABLE_HEADER_BG) > 0);
}

#[test]
fn workbench_table_tail_uses_deep_tail_surface() {
    let bytes = paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableTail", false)]),
    );

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_TAIL_BG);
    assert!(different_pixel_count(&bytes, 240, 14, 11, 90, 14, TABLE_TAIL_BG) > 0);
}
