use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::style_selector::{
    WORKBENCH_TABLE_HEADER_BG as TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT as TABLE_HEADER_TEXT,
    WORKBENCH_TABLE_HOVER_BG as TABLE_HOVER_BG, WORKBENCH_TABLE_SELECTED_BG as TABLE_SELECTED_BG,
    WORKBENCH_TABLE_SEPARATOR as TABLE_SEPARATOR, WORKBENCH_TABLE_TAIL_BG as TABLE_TAIL_BG,
};
use super::cells::{split_legacy_table_text, table_cell_rect, table_cells};
use super::style::{table_cell_color, table_row_style};
use super::surface::table_paint_rect;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn table_cells_prefer_declared_options_over_legacy_text() {
    let node = TemplatePaneNodeData {
        text: "Legacy Row".into(),
        options: model_rc(vec![
            SharedString::from("Item_02"),
            SharedString::from("Material"),
            SharedString::from("512 KB"),
            SharedString::from("10m ago"),
        ]),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        table_cells(&node),
        vec!["Item_02", "Material", "512 KB", "10m ago"]
    );
}

#[test]
fn legacy_table_text_keeps_size_and_modified_units_together() {
    assert_eq!(
        split_legacy_table_text("Item_03     Texture     1.2 MB      1h ago"),
        vec!["Item_03", "Texture", "1.2 MB", "1h ago"]
    );
}

#[test]
fn workbench_table_row_paints_selected_surface_and_action_glyph() {
    let bytes = super::super::template_nodes::paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableSelected", true)]),
    );

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_SELECTED_BG);
    assert!(different_pixel_count(&bytes, 240, 220, 15, 14, 14, TABLE_SELECTED_BG) > 0);
    assert!(different_pixel_count(&bytes, 240, 14, 11, 90, 14, TABLE_SELECTED_BG) > 0);
}

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
fn workbench_table_header_paints_muted_surface_separator_and_gear() {
    let node = table_node("WorkbenchTableHeader", false);
    assert_eq!(table_cell_color(&node, 0), TABLE_HEADER_TEXT);
    let bytes =
        super::super::template_nodes::paint_template_nodes_for_test(240, 44, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_HEADER_BG);
    assert_eq!(pixel_at(&bytes, 240, 8, 31), TABLE_SEPARATOR);
    assert!(different_pixel_count(&bytes, 240, 220, 15, 14, 14, TABLE_HEADER_BG) > 0);
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
fn workbench_table_row_style_uses_shared_state_priority() {
    let mut node = table_node("WorkbenchTableRowRoot", false);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let pressed = table_row_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, PALETTE.surface_pressed);
    assert_eq!(pressed.border, Some(PALETTE.focus_ring));
    assert_eq!(pressed.text_for_cell(0), PALETTE.text);

    node.pressed = false;
    let focused = table_row_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, TABLE_HOVER_BG);
    assert_eq!(focused.border, Some(PALETTE.focus_ring));

    node.disabled = true;
    let disabled = table_row_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, PALETTE.surface_disabled);
    assert_eq!(disabled.border, None);
    assert_eq!(disabled.text_for_cell(0), PALETTE.text_disabled);
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
fn workbench_table_tail_uses_deep_tail_surface() {
    let bytes = super::super::template_nodes::paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableTail", false)]),
    );

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_TAIL_BG);
    assert!(different_pixel_count(&bytes, 240, 14, 11, 90, 14, TABLE_TAIL_BG) > 0);
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

fn table_node(control_id: &str, selected: bool) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Table".into(),
        options: model_rc(vec![
            SharedString::from("Item_02"),
            SharedString::from("Material"),
            SharedString::from("512 KB"),
            SharedString::from("10m ago"),
        ]),
        selected,
        frame: super::super::super::data::TemplateNodeFrameData {
            x: 4.0,
            y: 4.0,
            width: 232.0,
            height: 28.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn different_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    reference: [u8; 4],
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            if pixel_at(bytes, frame_width, px, py) != reference {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
