mod actions;
mod cells;
mod identity;
mod style;
mod surface;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use actions::push_table_action;
use cells::{push_table_cells, table_cells};
use identity::{is_table_row, is_workbench_table_row};
use surface::{push_table_row_surface, table_paint_rect};

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

#[cfg(test)]
#[path = "template_table_rows_tests.rs"]
mod tests;
