use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::cells::{push_table_cells, table_cell_rect};
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

    let baseline = table_cell_rect(&table_node("WorkbenchTableHeader", false), &rect, 0);
    let cell_rect = table_cell_rect(&node, &rect, 0);
    assert_eq!(cell_rect.x, baseline.x - 1.0);
    assert_eq!(cell_rect.y, baseline.y + 3.0);
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

    let baseline = table_cell_rect(&table_node("WorkbenchTableRowRoot", false), &rect, 0);
    let first_cell = table_cell_rect(&node, &rect, 0);
    let second_cell = table_cell_rect(&node, &rect, 1);

    assert_eq!(first_cell.x, baseline.x + 4.0);
    assert!(second_cell.x > first_cell.x);
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
    let baseline = table_cell_rect(&table_node("WorkbenchTableTail", false), &row_rect, 0);
    let first_cell = table_cell_rect(&node, &row_rect, 0);
    let second_cell = table_cell_rect(&node, &row_rect, 1);
    let fourth_cell = table_cell_rect(&node, &row_rect, 3);

    assert_eq!(row_rect.y, 4.5);
    assert_eq!(first_cell.x, baseline.x + 6.0);
    assert_eq!(first_cell.y, 8.0);
    assert!(second_cell.x > first_cell.x);
    assert!(fourth_cell.width <= 0.0 || fourth_cell.x > second_cell.x);
    assert_eq!(table_cell_color(&node, 3), [170, 181, 186, 255]);
}

#[test]
fn table_columns_respect_readable_minimums_when_width_allows() {
    let node = table_node("WorkbenchTableHeader", false);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 360.0,
        height: 28.0,
    };

    let first_cell = table_cell_rect(&node, &rect, 0);
    let second_cell = table_cell_rect(&node, &rect, 1);
    let third_cell = table_cell_rect(&node, &rect, 2);
    let fourth_cell = table_cell_rect(&node, &rect, 3);

    assert!(first_cell.width >= 120.0);
    assert!(second_cell.width >= 56.0);
    assert!(third_cell.width >= 56.0);
    assert!(fourth_cell.width >= 72.0);
}

#[test]
fn table_columns_drop_low_priority_numeric_cells_when_too_narrow() {
    let node = table_node("WorkbenchTableHeader", false);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };
    let cells = vec![
        "Name".to_string(),
        "Type".to_string(),
        "Size".to_string(),
        "Rev".to_string(),
    ];
    let mut commands = Vec::new();

    push_table_cells(&mut commands, &node, &rect, &rect, 0, 1.0, &cells);

    let rendered_text = commands
        .iter()
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(rendered_text, vec!["Name", "Type"]);
}

#[test]
fn table_columns_drop_numeric_cells_for_narrow_layout_context() {
    let node = TemplatePaneNodeData {
        component_variant: "asset-table layoutNarrow".into(),
        ..table_node("WorkbenchTableHeader", false)
    };
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 360.0,
        height: 28.0,
    };
    let cells = vec![
        "Name".to_string(),
        "Type".to_string(),
        "Size".to_string(),
        "Rev".to_string(),
    ];
    let mut commands = Vec::new();

    push_table_cells(&mut commands, &node, &rect, &rect, 0, 1.0, &cells);

    let rendered_text = commands
        .iter()
        .filter_map(|command| command.text.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(rendered_text, vec!["Name", "Type"]);
}

#[test]
fn numeric_table_columns_align_text_to_the_right_edge() {
    let node = table_node("WorkbenchTableRowRoot", false);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 360.0,
        height: 28.0,
    };
    let cells = vec![
        "Item_02".to_string(),
        "Material".to_string(),
        "1K".to_string(),
        "r42".to_string(),
    ];
    let mut commands = Vec::new();

    push_table_cells(&mut commands, &node, &rect, &rect, 0, 1.0, &cells);

    let size_cell = table_cell_rect(&node, &rect, 2);
    let size_text = &commands[2];
    assert!(size_text.frame.x > size_cell.x);
    assert!(
        ((size_text.frame.x + size_text.frame.width) - (size_cell.x + size_cell.width)).abs()
            < 0.01
    );
}
