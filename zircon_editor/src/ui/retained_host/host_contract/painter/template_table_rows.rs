use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_any_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_table_row_style, WorkbenchTableRowStyle};
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TABLE_CELL_FONT_SIZE: f32 = 11.0;
const TABLE_CELL_INSET_X: f32 = 9.0;
const TABLE_CELL_INSET_Y: f32 = 4.0;
const TABLE_ACTION_WIDTH: f32 = 24.0;
const TABLE_ROW_RADIUS: f32 = 3.0;
const TABLE_COLUMN_RATIOS: [f32; 4] = [0.36, 0.27, 0.19, 0.18];

pub(super) fn push_table_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_table_row(node) {
        return false;
    }

    let cells = table_cells(node);
    if cells.is_empty() {
        return false;
    }

    let rect = table_paint_rect(node, rect);
    push_table_row_surface(commands, node, &rect, clip, order, opacity);
    push_table_cells(commands, node, &rect, clip, order + 2, opacity, &cells);
    push_table_action(commands, node, &rect, clip, order + 3, opacity);
    true
}

pub(super) fn push_table_row_text_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_table_row(node) {
        return false;
    }

    let cells = table_cells(node);
    if cells.is_empty() {
        return false;
    }

    push_table_cells(commands, node, rect, clip, order, opacity, &cells);
    true
}

fn push_table_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(table_row_background(node)),
        table_row_border(node),
        table_row_border_width(node),
        TABLE_ROW_RADIUS,
        opacity,
    ));

    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - 1.0).max(0.0),
            width: rect.width,
            height: 1.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(table_row_style(node).separator),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn table_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    if is_table_tail(node) || is_table_selected(node) {
        FrameRect {
            x: rect.x + node.layout_offset_x,
            y: rect.y + node.layout_offset_y,
            width: rect.width,
            height: rect.height,
        }
    } else {
        rect.clone()
    }
}

fn push_table_cells(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    cells: &[String],
) {
    for (index, cell) in cells.iter().take(TABLE_COLUMN_RATIOS.len()).enumerate() {
        commands.push(text_command(
            table_cell_rect(node, rect, index),
            clip,
            order,
            cell,
            table_cell_color(node, index),
            opacity,
        ));
    }
}

fn push_table_action(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let action_rect = FrameRect {
        x: rect.x + rect.width - TABLE_ACTION_WIDTH + 7.0 + content_offset_x,
        y: rect.y + (rect.height - 14.0).max(0.0) * 0.5 + content_offset_y,
        width: 14.0,
        height: 14.0,
    };
    let action_color = table_row_style(node).action;
    if is_table_header(node) {
        push_table_gear(commands, &action_rect, clip, order, action_color, opacity);
    } else {
        push_table_kebab(commands, &action_rect, clip, order, action_color, opacity);
    }
}

fn is_table_row(node: &TemplatePaneNodeData) -> bool {
    is_any_component_family(
        node,
        &[
            TemplateComponentFamily::Table,
            TemplateComponentFamily::TableRow,
        ],
    )
}

fn is_workbench_table_row(node: &TemplatePaneNodeData) -> bool {
    uses_workbench_visual_language(node)
        && is_any_component_family(
            node,
            &[
                TemplateComponentFamily::Table,
                TemplateComponentFamily::TableRow,
            ],
        )
}

fn table_cells(node: &TemplatePaneNodeData) -> Vec<String> {
    let option_cells = (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|cell| cell.to_string())
        .filter(|cell| !cell.trim().is_empty())
        .collect::<Vec<_>>();
    if !option_cells.is_empty() {
        return option_cells;
    }
    split_legacy_table_text(node.text.as_str())
}

fn split_legacy_table_text(text: &str) -> Vec<String> {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        [] => Vec::new(),
        [name, kind, size, size_unit, modified_value, modified_unit, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            format!("{size} {size_unit}"),
            format!("{modified_value} {modified_unit}"),
        ],
        [name, kind, size, modified, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            (*size).to_string(),
            (*modified).to_string(),
        ],
        _ => vec![text.trim().to_string()],
    }
}

fn table_cell_rect(node: &TemplatePaneNodeData, rect: &FrameRect, index: usize) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let mut x = rect.x + TABLE_CELL_INSET_X + content_offset_x;
    let available_width = (rect.width - TABLE_CELL_INSET_X * 2.0 - TABLE_ACTION_WIDTH).max(1.0);
    for ratio in TABLE_COLUMN_RATIOS.iter().take(index) {
        x += available_width * ratio;
    }
    let width = TABLE_COLUMN_RATIOS
        .get(index)
        .map(|ratio| available_width * ratio)
        .unwrap_or(available_width)
        .max(1.0);
    FrameRect {
        x: x + table_cell_offset_x(node, index),
        y: rect.y + TABLE_CELL_INSET_Y + content_offset_y,
        width: width.max(1.0),
        height: (rect.height - TABLE_CELL_INSET_Y * 2.0).max(1.0),
    }
}

fn table_content_offset(node: &TemplatePaneNodeData) -> (f32, f32) {
    if is_table_header(node) || is_table_tail(node) {
        (node.layout_content_offset_x, node.layout_content_offset_y)
    } else {
        (0.0, 0.0)
    }
}

fn table_cell_offset_x(node: &TemplatePaneNodeData, index: usize) -> f32 {
    match index {
        0 => node.layout_first_cell_offset_x,
        1 => node.layout_second_cell_offset_x,
        2 => node.layout_third_cell_offset_x,
        3 => node.layout_fourth_cell_offset_x,
        _ => 0.0,
    }
}

fn table_cell_color(node: &TemplatePaneNodeData, index: usize) -> [u8; 4] {
    table_row_style(node).text_for_cell(index)
}

fn table_row_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    table_row_style(node).background
}

fn table_row_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    table_row_style(node).border
}

fn table_row_border_width(node: &TemplatePaneNodeData) -> f32 {
    table_row_style(node).border_width
}

fn table_row_style(node: &TemplatePaneNodeData) -> WorkbenchTableRowStyle {
    select_workbench_table_row_style(node)
}

fn is_table_header(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchTableHeader"
}

fn is_table_tail(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchTableTail"
}

fn is_table_selected(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchTableSelected"
}

fn text_command(
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) -> HostPaintCommand {
    HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        TABLE_CELL_FONT_SIZE,
        TABLE_CELL_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    )
}

fn push_table_kebab(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for y in [3.0, 6.0, 9.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + 6.0,
                y: rect.y + y,
                width: 2.0,
                height: 2.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_table_gear(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 2.0,
            width: 6.0,
            height: 1.0,
        },
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 11.0,
            width: 6.0,
            height: 1.0,
        },
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 4.0,
            width: 1.0,
            height: 6.0,
        },
        FrameRect {
            x: rect.x + 11.0,
            y: rect.y + 4.0,
            width: 1.0,
            height: 6.0,
        },
        FrameRect {
            x: rect.x + 6.0,
            y: rect.y + 6.0,
            width: 2.0,
            height: 2.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::super::style_selector::{
        WORKBENCH_TABLE_HEADER_BG as TABLE_HEADER_BG,
        WORKBENCH_TABLE_HEADER_TEXT as TABLE_HEADER_TEXT,
        WORKBENCH_TABLE_HOVER_BG as TABLE_HOVER_BG, WORKBENCH_TABLE_ROW_BG as TABLE_ROW_BG,
        WORKBENCH_TABLE_SELECTED_BG as TABLE_SELECTED_BG,
        WORKBENCH_TABLE_SEPARATOR as TABLE_SEPARATOR, WORKBENCH_TABLE_TAIL_BG as TABLE_TAIL_BG,
    };
    use super::*;
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
        let bytes = super::super::template_nodes::paint_template_nodes_for_test(
            240,
            44,
            model_rc(vec![node]),
        );

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
}
