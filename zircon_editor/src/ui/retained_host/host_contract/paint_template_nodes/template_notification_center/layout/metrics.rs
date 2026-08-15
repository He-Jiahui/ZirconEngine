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
        panel_radius: metrics.radius_control,
        row_radius: metrics.radius_control,
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
mod tests {
    use super::super::super::super::super::paint_theme::METRICS;
    use super::*;

    #[test]
    fn notification_metrics_project_from_shared_host_control_tokens() {
        let mut host = METRICS;
        host.radius_control = 5.0;
        host.border_width = 1.5;
        host.font_small = 10.0;
        host.font_body = 12.0;
        host.line_height_ratio = 1.25;
        host.input_pad = [7.0, 7.0, 2.0, 3.0];
        host.button_pad_x = 14.0;
        host.gap_s = 3.0;
        host.gap_m = 9.0;
        host.gap_l = 13.0;
        host.row_height = 30.0;
        host.selection_indicator_width = 2.0;

        let notification = notification_center_metrics_from_host(host);

        assert_eq!(notification.panel_radius, 5.0);
        assert_eq!(notification.row_radius, 5.0);
        assert_eq!(notification.border_width, 1.5);
        assert_eq!(notification.header_font_size, 12.0);
        assert_eq!(notification.header_line_height, 15.0);
        assert_eq!(notification.message_font_size, 10.0);
        assert_eq!(notification.message_line_height, 12.5);
        assert_eq!(notification.title_font_size, 12.0);
        assert_eq!(notification.title_line_height, 15.0);
        assert_eq!(notification.header_top, 12.0);
        assert_eq!(notification.panel_padding_x, 14.0);
        assert_eq!(notification.row_gap, 6.0);
        assert_eq!(notification.row_height, 45.0);
        assert_eq!(notification.row_inset_x, 9.0);
        assert_eq!(notification.row_top, 39.0);
        assert_eq!(notification.mark_left, 12.0);
        assert_eq!(notification.mark_width, 3.5);
        assert_eq!(notification.text_left, 26.0);
        assert_eq!(notification.text_right_inset, 14.0);
        assert_eq!(notification.title_top, 5.0);
        assert_eq!(notification.message_top, 23.0);
    }
}
