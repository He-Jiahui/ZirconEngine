use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::actions::push_table_action;
use super::cells::{push_table_cells, table_cells};
use super::geometry::has_paintable_table_row_extent;
use super::identity::{is_table_row, is_workbench_table_row};
use super::layers::{action_slot_order, cells_order};
use super::surface::{push_table_row_surface, table_paint_rect};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_row_commands(
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
    if !has_paintable_table_row_extent(&rect) {
        return true;
    }
    push_table_row_surface(commands, node, &rect, clip, order, opacity);
    push_table_cells(
        commands,
        node,
        &rect,
        clip,
        cells_order(order),
        opacity,
        &cells,
    );
    push_table_action(
        commands,
        node,
        &rect,
        clip,
        action_slot_order(order),
        opacity,
    );
    true
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_row_text_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_workbench_table_row(node) || !is_table_row(node) {
        return false;
    }

    let cells = table_cells(node);
    if cells.is_empty() {
        return false;
    }

    push_table_cells(commands, node, rect, clip, order, opacity, &cells);
    true
}
