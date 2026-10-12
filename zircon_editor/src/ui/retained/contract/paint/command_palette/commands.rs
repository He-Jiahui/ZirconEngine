//! 命令面板的可见行提交入口。节点已在宿主坐标系中，clip 是本次允许绘制的区域。
//! 仅为可见行和少量预取行预算命令容量，避免大目录在每帧遍历或复制全部行。

use std::ops::Range;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::is_command_palette;
use super::layers::{empty_message_order, row_order};
use super::layout::{command_palette_metrics, min_frame_extent, paint_rect, row_rect};
use super::panel::{push_command_palette_empty_message, push_command_palette_panel_commands};
use super::rows::push_command_row_commands;

const COMMAND_PALETTE_PANEL_COMMAND_UPPER_BOUND: usize = 4;
const COMMAND_PALETTE_EMPTY_COMMAND_UPPER_BOUND: usize = 1;
const COMMAND_PALETTE_ROW_COMMAND_UPPER_BOUND: usize = 4;

/// 由专用节点分派调用；false 表示不是本组件，true 表示已接管，包括关闭或尺寸不足的状态。
/// 调用方应停止通用 fallback，避免给隐藏面板补画背景；不在这里执行任何命令。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_command_palette(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = paint_rect(rect);
    let min_frame_extent = min_frame_extent();
    if rect.width <= min_frame_extent || rect.height <= min_frame_extent {
        return true;
    }

    let row_count = node.structured_options.row_count();
    let visible_rows = command_palette_visible_rows(&rect, clip, row_count);
    let body_command_upper_bound = if row_count == 0 {
        COMMAND_PALETTE_EMPTY_COMMAND_UPPER_BOUND
    } else {
        visible_rows
            .len()
            .saturating_mul(COMMAND_PALETTE_ROW_COMMAND_UPPER_BOUND)
    };
    let command_upper_bound =
        COMMAND_PALETTE_PANEL_COMMAND_UPPER_BOUND.saturating_add(body_command_upper_bound);
    commands.reserve(command_upper_bound);

    push_command_palette_panel_commands(commands, node, &rect, clip, order, opacity);

    if row_count == 0 {
        push_command_palette_empty_message(
            commands,
            &rect,
            clip,
            empty_message_order(order),
            opacity,
        );
        return true;
    }

    for row in visible_rows {
        let Some(option) = node.structured_options.get(row) else {
            continue;
        };
        push_command_row_commands(
            commands,
            option,
            &row_rect(&rect, row),
            clip,
            row_order(order, row),
            opacity,
        );
    }

    true
}

const COMMAND_PALETTE_PAINT_OVERSCAN_ROWS: usize = 1;

/// 从当前面板和裁剪范围选择绘制索引；行号仍是完整候选列表的绝对索引。
/// 这里不负责滚动位移或分页，调用方必须传入已经投影后的面板位置。
fn command_palette_visible_rows(
    panel: &FrameRect,
    clip: &FrameRect,
    row_count: usize,
) -> Range<usize> {
    if row_count == 0
        || panel.width <= 0.0
        || panel.height <= 0.0
        || clip.width <= 0.0
        || clip.height <= 0.0
        || clip.x >= panel.x + panel.width
        || clip.x + clip.width <= panel.x
    {
        return 0..0;
    }

    let metrics = command_palette_metrics();
    let list_top = panel.y + metrics.list_top;
    let list_bottom =
        (list_top + row_count as f32 * metrics.row_height).min(panel.y + panel.height);
    let visible_top = clip.y.max(list_top);
    let visible_bottom = (clip.y + clip.height).min(list_bottom);
    if visible_bottom <= visible_top {
        return 0..0;
    }

    let first_visible = ((visible_top - list_top) / metrics.row_height)
        .floor()
        .max(0.0) as usize;
    let visible_end = ((visible_bottom - list_top) / metrics.row_height)
        .ceil()
        .max(0.0) as usize;
    let visible_end = visible_end.min(row_count);
    if first_visible >= visible_end {
        return 0..0;
    }

    first_visible.saturating_sub(COMMAND_PALETTE_PAINT_OVERSCAN_ROWS)
        ..visible_end
            .saturating_add(COMMAND_PALETTE_PAINT_OVERSCAN_ROWS)
            .min(row_count)
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
