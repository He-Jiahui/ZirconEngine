use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchAlertMetrics {
    pub border_width: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_size: f32,
    pub icon_left: f32,
    pub text_gap: f32,
    pub text_vertical_inset: f32,
    pub text_right_inset: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_metrics(
) -> WorkbenchAlertMetrics {
    alert_metrics_from_host(current_host_metrics())
}

pub(super) fn alert_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchAlertMetrics {
    let icon_left = (metrics.gap_l - metrics.border_width * 2.0).max(0.0);

    WorkbenchAlertMetrics {
        border_width: metrics.border_width,
        radius: metrics.radius_control + metrics.gap_s,
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        icon_size: (metrics.row_height - metrics.gap_m - metrics.border_width * 2.0)
            .max(metrics.font_body),
        icon_left,
        text_gap: metrics.gap_m,
        text_vertical_inset: metrics.gap_s + metrics.border_width,
        text_right_inset: icon_left,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchToastMetrics {
    pub border_width: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_size: f32,
    pub icon_left: f32,
    pub text_gap: f32,
    pub trailing_inset: f32,
    pub close_size: f32,
    pub action_gap: f32,
    pub action_width: f32,
    pub action_minimum_width: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_metrics(
) -> WorkbenchToastMetrics {
    toast_metrics_from_host(current_host_metrics())
}

pub(super) fn toast_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchToastMetrics {
    let font_size = metrics.font_small;
    let icon_size =
        (metrics.row_height - metrics.gap_m - metrics.border_width * 2.0).max(font_size);
    let icon_left = metrics.gap_l;
    let text_gap = metrics.gap_m + metrics.border_width;
    let trailing_inset = (metrics.gap_l - metrics.border_width * 2.0).max(0.0);
    let close_size = metrics.row_height * 0.5;
    let action_gap = metrics.gap_s;
    let action_width = font_size * 4.0 + metrics.border_width;
    let minimum_text_width = metrics.row_height * 3.5;
    // Keep a compact message budget before adding the action and close affordances.
    let action_minimum_width = icon_left
        + icon_size
        + text_gap
        + minimum_text_width
        + action_gap
        + action_width
        + close_size
        + trailing_inset;

    WorkbenchToastMetrics {
        border_width: metrics.border_width,
        radius: metrics.radius_control + metrics.gap_s,
        font_size,
        line_height: metrics.line_height(font_size),
        icon_size,
        icon_left,
        text_gap,
        trailing_inset,
        close_size,
        action_gap,
        action_width,
        action_minimum_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn alert_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.border_width = 2.0;
        host.radius_control = 3.0;
        host.font_body = 12.5;
        host.line_height_ratio = 1.4;
        host.gap_s = 5.0;
        host.gap_m = 9.0;
        host.gap_l = 14.0;
        host.row_height = 34.0;

        let alert = alert_metrics_from_host(host);

        assert_eq!(alert.border_width, 2.0);
        assert_eq!(alert.radius, 8.0);
        assert_eq!(alert.font_size, 12.5);
        assert!((alert.line_height - 17.5).abs() < f32::EPSILON);
        assert_eq!(alert.icon_size, 21.0);
        assert_eq!(alert.icon_left, 10.0);
        assert_eq!(alert.text_gap, 9.0);
        assert_eq!(alert.text_vertical_inset, host.gap_s + host.border_width);
        assert_eq!(alert.text_right_inset, 10.0);
    }

    #[test]
    fn toast_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.border_width = 2.0;
        host.radius_control = 3.0;
        host.font_small = 11.0;
        host.line_height_ratio = 1.3;
        host.gap_s = 5.0;
        host.gap_m = 9.0;
        host.gap_l = 13.0;
        host.row_height = 32.0;

        let toast = toast_metrics_from_host(host);

        assert_eq!(toast.border_width, 2.0);
        assert_eq!(toast.radius, 8.0);
        assert_eq!(toast.font_size, 11.0);
        assert!((toast.line_height - 14.3).abs() < f32::EPSILON);
        assert_eq!(toast.icon_size, 19.0);
        assert_eq!(toast.icon_left, 13.0);
        assert_eq!(toast.text_gap, 11.0);
        assert_eq!(toast.trailing_inset, 9.0);
        assert_eq!(toast.close_size, 16.0);
        assert_eq!(toast.action_gap, 5.0);
        assert_eq!(toast.action_width, 46.0);
        assert_eq!(toast.action_minimum_width, 231.0);
    }
}
