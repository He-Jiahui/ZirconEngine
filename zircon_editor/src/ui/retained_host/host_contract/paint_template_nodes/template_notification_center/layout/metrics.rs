//! 把宿主控件密度投影成通知面板和记录行的共同尺寸合同。
//! 预取计算与实际行矩形必须使用同一行高/间距，避免滚动clip附近遗漏记录。

use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct NotificationCenterMetrics
{
    pub border_width: f32,
    pub header_font_size: f32,
    pub header_line_height: f32,
    pub mark_radius: f32,
    pub message_font_size: f32,
    pub message_line_height: f32,
    pub panel_radius: f32,
    pub row_radius: f32,
    pub title_font_size: f32,
    pub title_line_height: f32,
    pub empty_text_top: f32,
    pub header_top: f32,
    pub panel_padding_x: f32,
    pub row_gap: f32,
    pub row_height: f32,
    pub row_inset_x: f32,
    pub row_top: f32,
    pub mark_height: f32,
    pub mark_left: f32,
    pub mark_top: f32,
    pub mark_width: f32,
    pub text_left: f32,
    pub text_right_inset: f32,
    pub title_top: f32,
    pub message_top: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn notification_center_metrics(
) -> NotificationCenterMetrics {
    notification_center_metrics_from_host(current_host_metrics())
}

fn notification_center_metrics_from_host(metrics: HostControlMetrics) -> NotificationCenterMetrics {
    let header_font_size = metrics.font_body;
    let header_line_height = metrics.line_height(header_font_size);
    let message_font_size = metrics.font_small;
    let message_line_height = metrics.line_height(message_font_size);
    let title_font_size = metrics.font_body;
    let title_line_height = metrics.line_height(title_font_size);
    let header_top = metrics.gap_m + metrics.border_width * 2.0;
    let title_top = metrics.input_pad[2] + metrics.gap_s;
    let row_height =
        (title_top + title_line_height + metrics.gap_s + message_line_height + metrics.gap_m)
            .round();
    let mark_left = metrics.gap_m + metrics.border_width * 2.0;
    let mark_width = metrics.selection_indicator_width + metrics.border_width;

    NotificationCenterMetrics {
        border_width: metrics.border_width,
        header_font_size,
        header_line_height,
        mark_radius: metrics.border_width,
        message_font_size,
        message_line_height,
        panel_radius: metrics.radius_control + metrics.gap_s,
        row_radius: (metrics.radius_control - metrics.border_width * 2.0).max(0.0),
        title_font_size,
        title_line_height,
        empty_text_top: metrics.row_height + metrics.gap_l + metrics.gap_m,
        header_top,
        panel_padding_x: metrics.button_pad_x,
        row_gap: metrics.gap_s + metrics.border_width * 2.0,
        row_height,
        row_inset_x: metrics.gap_m,
        row_top: header_top + header_line_height + metrics.gap_m + metrics.border_width * 2.0,
        mark_height: (row_height - metrics.gap_m * 2.0).max(0.0),
        mark_left,
        mark_top: metrics.gap_m,
        mark_width,
        text_left: mark_left + mark_width + metrics.gap_m + metrics.border_width,
        text_right_inset: metrics.button_pad_x,
        title_top,
        message_top: title_top + title_line_height + metrics.gap_s,
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
