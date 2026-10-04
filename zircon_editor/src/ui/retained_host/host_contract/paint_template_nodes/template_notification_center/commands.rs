//! 通知中心的绘制分派与可见行筛选入口。关闭时立即返回接管状态，避免解析主题、行数或文本。
//! 打开后只遍历clip附近的行及一行overscan，选项数据仍按完整列表的绝对索引借用。

use std::ops::Range;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::is_notification_center;
#[cfg(test)]
use super::instrumentation::{
    record_metrics_resolution, record_palette_resolution, record_row_count_read, record_row_visit,
};
use super::layout::{notification_center_metrics, paint_rect, row_rect};
use super::panel::{push_empty_notification_message, push_notification_panel_commands};
use super::row::push_notification_row;
use super::style::current_notification_center_palette;

/// 由专用节点分派调用；false 表示交其他painter，true 包含关闭和退化尺寸状态。
/// 调用方必须传入投影后的节点矩形与相同坐标系的clip，且不能在true后补画通用fallback。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_center_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_notification_center(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = paint_rect(rect);
    if rect.width <= 1.0 || rect.height <= 1.0 {
        return true;
    }

    #[cfg(test)]
    record_palette_resolution();
    let palette = current_notification_center_palette();
    #[cfg(test)]
    record_metrics_resolution();
    let metrics = notification_center_metrics_for_node(node);
    push_notification_panel_commands(
        commands, node, &rect, clip, order, opacity, palette, &metrics,
    );

    #[cfg(test)]
    record_row_count_read();
    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        push_empty_notification_message(
            commands,
            node,
            &rect,
            clip,
            order + 2,
            opacity,
            palette,
            &metrics,
        );
        return true;
    }

    for row in notification_center_visible_rows(&rect, clip, row_count, &metrics) {
        let Some(option) = node.structured_options.get(row) else {
            continue;
        };
        #[cfg(test)]
        record_row_visit();
        push_notification_row(
            commands,
            option,
            &row_rect(&rect, row, &metrics),
            clip,
            order + 3 + row as i32 * 4,
            opacity,
            palette,
            &metrics,
        );
    }

    true
}

fn notification_center_metrics_for_node(
    node: &TemplatePaneNodeData,
) -> super::layout::NotificationCenterMetrics {
    let mut metrics = notification_center_metrics();
    if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
        metrics.panel_radius = node.corner_radius;
    }
    metrics
}

const NOTIFICATION_CENTER_PAINT_OVERSCAN_ROWS: usize = 1;

/// 根据当前面板与clip求候选绝对索引，并保留相邻一行用于滚动边界。
/// 仅决定本帧的绘制访问，不负责通知列表的可见数量上限或滚动状态。
fn notification_center_visible_rows(
    panel: &FrameRect,
    clip: &FrameRect,
    row_count: usize,
    metrics: &super::layout::NotificationCenterMetrics,
) -> Range<usize> {
    let geometry = [
        panel.x,
        panel.y,
        panel.width,
        panel.height,
        clip.x,
        clip.y,
        clip.width,
        clip.height,
    ];
    if row_count == 0
        || geometry.into_iter().any(|value| !value.is_finite())
        || panel.width <= 0.0
        || panel.height <= 0.0
        || clip.width <= 0.0
        || clip.height <= 0.0
        || clip.x >= panel.x + panel.width
        || clip.x + clip.width <= panel.x
    {
        return 0..0;
    }

    let row_stride = metrics.row_height + metrics.row_gap;
    if !row_stride.is_finite() || row_stride <= 0.0 || metrics.row_height <= 0.0 {
        return 0..0;
    }

    let list_top = panel.y + metrics.row_top;
    let content_bottom =
        list_top + row_count.saturating_sub(1) as f32 * row_stride + metrics.row_height;
    let list_bottom = content_bottom.min(panel.y + panel.height);
    let visible_top = clip.y.max(list_top);
    let visible_bottom = (clip.y + clip.height).min(list_bottom);
    if visible_bottom <= visible_top {
        return 0..0;
    }

    let first_visible =
        (((visible_top - list_top - metrics.row_height) / row_stride).floor() as isize + 1).max(0)
            as usize;
    let visible_end = ((visible_bottom - list_top) / row_stride).ceil().max(0.0) as usize;
    let visible_end = visible_end.min(row_count);
    if first_visible >= visible_end {
        return 0..0;
    }

    first_visible.saturating_sub(NOTIFICATION_CENTER_PAINT_OVERSCAN_ROWS)
        ..visible_end
            .saturating_add(NOTIFICATION_CENTER_PAINT_OVERSCAN_ROWS)
            .min(row_count)
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
