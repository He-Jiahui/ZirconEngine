//! 把绝对记录索引映射到面板行；标记、标题和消息都限制在该行区域内。
//! 消息与标题共享横向宽度、纵向分带；调用方只提交与当前clip交集的行。

use super::super::super::super::data::FrameRect;
use super::metrics::NotificationCenterMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_rect(
    panel_rect: &FrameRect,
    row: usize,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let horizontal_inset = metrics.row_inset_x.min(panel_rect.width.max(0.0) * 0.5);
    let panel_bottom = panel_rect.y + panel_rect.height.max(0.0);
    let y = (panel_rect.y + metrics.row_top + row as f32 * (metrics.row_height + metrics.row_gap))
        .min(panel_bottom);
    FrameRect {
        x: panel_rect.x + horizontal_inset,
        y,
        width: (panel_rect.width - horizontal_inset * 2.0).max(0.0),
        height: metrics.row_height.min((panel_bottom - y).max(0.0)),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn mark_rect(
    row_rect: &FrameRect,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let x = row_rect.x + metrics.mark_left.min(row_rect.width.max(0.0));
    let y = row_rect.y + metrics.mark_top.min(row_rect.height.max(0.0));
    FrameRect {
        x,
        y,
        width: metrics
            .mark_width
            .min((row_rect.x + row_rect.width - x).max(0.0)),
        height: metrics
            .mark_height
            .min((row_rect.y + row_rect.height - y).max(0.0)),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_rect(
    row_rect: &FrameRect,
    width: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    row_text_rect(
        row_rect,
        metrics.title_top,
        width,
        metrics.title_line_height,
        metrics,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn message_rect(
    row_rect: &FrameRect,
    width: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    row_text_rect(
        row_rect,
        metrics.message_top,
        width,
        metrics.message_line_height,
        metrics,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_text_width(
    row_rect: &FrameRect,
    metrics: &NotificationCenterMetrics,
) -> f32 {
    let left = row_text_left(row_rect, metrics);
    (row_rect.x + row_rect.width - metrics.text_right_inset - left).max(0.0)
}

fn row_text_left(row_rect: &FrameRect, metrics: &NotificationCenterMetrics) -> f32 {
    row_rect.x + metrics.text_left.min(row_rect.width.max(0.0))
}

fn row_text_rect(
    row_rect: &FrameRect,
    y_offset: f32,
    width: f32,
    height: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let x = row_text_left(row_rect, metrics);
    let y = row_rect.y + y_offset.min(row_rect.height.max(0.0));
    FrameRect {
        x,
        y,
        width: width.min((row_rect.x + row_rect.width - x).max(0.0)),
        height: height.min((row_rect.y + row_rect.height - y).max(0.0)),
    }
}

#[cfg(test)]
#[path = "tests/row.rs"]
mod tests;
