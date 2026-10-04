//! 通知标题与空态文案的位置预算；即使面板窄短也把文本区域限制在容器内。

use super::super::super::super::data::FrameRect;
use super::metrics::NotificationCenterMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn header_rect(
    panel_rect: &FrameRect,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    padded_panel_rect(
        panel_rect,
        metrics.header_top,
        metrics.header_line_height,
        metrics,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text_rect(
    panel_rect: &FrameRect,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    padded_panel_rect(
        panel_rect,
        metrics.empty_text_top,
        metrics.message_line_height,
        metrics,
    )
}

fn padded_panel_rect(
    panel_rect: &FrameRect,
    y_offset: f32,
    height: f32,
    metrics: &NotificationCenterMetrics,
) -> FrameRect {
    let horizontal_inset = metrics.panel_padding_x.min(panel_rect.width.max(0.0) * 0.5);
    let y = panel_rect.y + y_offset.min(panel_rect.height.max(0.0));
    FrameRect {
        x: panel_rect.x + horizontal_inset,
        y,
        width: (panel_rect.width - horizontal_inset * 2.0).max(0.0),
        height: height.min((panel_rect.y + panel_rect.height - y).max(0.0)),
    }
}

#[cfg(test)]
#[path = "tests/panel.rs"]
mod tests;
