use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::cells::table_cell_rect;
use super::super::style::table_cell_color;
use super::super::surface::table_paint_rect;
use super::support::table_node;

#[test]
fn workbench_table_selected_honors_declared_row_offset() {
    let node = TemplatePaneNodeData {
        layout_offset_x: -1.0,
        layout_offset_y: -1.5,
        ..table_node("WorkbenchTableSelected", true)
    };
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };

    let row_rect = table_paint_rect(&node, &rect);

    assert_eq!(row_rect.x, 3.0);
    assert_eq!(row_rect.y, 2.5);
}

#[test]
fn workbench_table_header_honors_content_offset_without_moving_row() {
    let node = TemplatePaneNodeData {
        layout_content_offset_x: -1.0,
        layout_content_offset_y: 3.0,
        ..table_node("WorkbenchTableHeader", false)
    };
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };

    let cell_rect = table_cell_rect(&node, &rect, 0);
    assert_eq!(cell_rect.x, 12.0);
    assert_eq!(cell_rect.y, 11.0);
    assert_eq!(node.frame.x, 4.0);
    assert_eq!(node.frame.y, 4.0);
}

#[test]
fn workbench_table_row_honors_declared_first_cell_offset() {
    let node = TemplatePaneNodeData {
        layout_first_cell_offset_x: 4.0,
        ..table_node("WorkbenchTableRowRoot", false)
    };
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };

    let first_cell = table_cell_rect(&node, &rect, 0);
    let second_cell = table_cell_rect(&node, &rect, 1);

    assert_eq!(first_cell.x, 17.0);
    assert!((second_cell.x - 81.4).abs() < 0.001);
}

#[test]
fn workbench_table_tail_honors_declared_content_and_cell_offsets() {
    let node = TemplatePaneNodeData {
        layout_offset_y: 0.5,
        layout_content_offset_y: -0.5,
        layout_first_cell_offset_x: 6.0,
        layout_second_cell_offset_x: 2.0,
        layout_fourth_cell_offset_x: -2.0,
        value_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(170, 181, 186),
        ..table_node("WorkbenchTableTail", false)
    };
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };

    let row_rect = table_paint_rect(&node, &rect);
    let first_cell = table_cell_rect(&node, &row_rect, 0);
    let second_cell = table_cell_rect(&node, &row_rect, 1);
    let fourth_cell = table_cell_rect(&node, &row_rect, 3);

    assert_eq!(row_rect.y, 4.5);
    assert_eq!(first_cell.x, 19.0);
    assert_eq!(first_cell.y, 8.0);
    assert!((second_cell.x - 83.4).abs() < 0.001);
    assert!((fourth_cell.x - 166.8).abs() < 0.001);
    assert_eq!(table_cell_color(&node, 3), [170, 181, 186, 255]);
}
