use super::super::super::super::data::FrameRect;
use super::common::fitted_centered_square;
use super::metrics::WorkbenchAlertMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_rect(
    rect: &FrameRect,
    metrics: WorkbenchAlertMetrics,
) -> FrameRect {
    fitted_centered_square(rect, metrics.icon_left, metrics.icon_size)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_text_rect(
    rect: &FrameRect,
    icon: Option<&FrameRect>,
    metrics: WorkbenchAlertMetrics,
) -> Option<FrameRect> {
    let text_left = icon
        .map(|icon| icon.x + icon.width + metrics.text_gap)
        .unwrap_or(rect.x + metrics.icon_left);
    let text_right = rect.x + rect.width - metrics.text_right_inset;
    (text_right > text_left).then(|| FrameRect {
        x: text_left,
        y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
        width: text_right - text_left,
        height: metrics.line_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn alert_geometry_uses_metric_derived_insets_and_line_height() {
        let mut host = METRICS;
        host.border_width = 2.0;
        host.font_body = 12.5;
        host.line_height_ratio = 1.4;
        host.gap_m = 9.0;
        host.gap_l = 14.0;
        host.row_height = 34.0;
        let metrics = super::super::metrics::alert_metrics_from_host(host);
        let rect = FrameRect {
            x: 5.0,
            y: 7.0,
            width: 200.0,
            height: 30.0,
        };

        let icon = alert_icon_rect(&rect, metrics);
        assert_eq!(icon.x, 15.0);
        assert_eq!(icon.y, 11.5);
        assert_eq!(icon.width, 21.0);

        assert_eq!(
            alert_text_rect(&rect, Some(&icon), metrics),
            Some(FrameRect {
                x: 45.0,
                y: 13.25,
                width: 150.0,
                height: 17.5,
            })
        );

        assert_eq!(
            alert_text_rect(&rect, None, metrics),
            Some(FrameRect {
                x: 15.0,
                y: 13.25,
                width: 180.0,
                height: 17.5,
            })
        );

        let compact = FrameRect {
            x: 3.0,
            y: 5.0,
            width: 14.0,
            height: 9.0,
        };
        let compact_icon = alert_icon_rect(&compact, metrics);
        assert_eq!(compact_icon.x, 13.0);
        assert_eq!(compact_icon.y, 7.5);
        assert_eq!(compact_icon.width, 4.0);
        assert_eq!(compact_icon.height, 4.0);
    }
}
