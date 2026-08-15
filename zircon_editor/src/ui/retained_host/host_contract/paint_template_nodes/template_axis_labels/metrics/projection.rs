use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::model::AxisLabelMetrics;

const LINK_LOBE_WIDTH_BORDER_UNITS: f32 = 2.0;
const LINK_OVERLAP_BORDER_UNITS: f32 = 2.0;
const LINK_LOBE_RADIUS_RATIO: f32 = 0.5;
const MIN_LINK_METRIC_EXTENT: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_metrics(
) -> AxisLabelMetrics {
    axis_label_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_metrics_from_host(
    metrics: HostControlMetrics,
) -> AxisLabelMetrics {
    let font_size = metrics.font_body + metrics.border_width;
    let link_lobe_width = (metrics.gap_m - metrics.border_width * LINK_LOBE_WIDTH_BORDER_UNITS)
        .max(MIN_LINK_METRIC_EXTENT);
    let link_lobe_height = (metrics.gap_m - metrics.border_width).max(MIN_LINK_METRIC_EXTENT);
    AxisLabelMetrics {
        font_size,
        line_height: metrics.line_height(font_size),
        link_lobe_width,
        link_lobe_height,
        link_lobe_radius: link_lobe_width * LINK_LOBE_RADIUS_RATIO,
        link_overlap: (metrics.border_width * LINK_OVERLAP_BORDER_UNITS)
            .max(MIN_LINK_METRIC_EXTENT),
        link_connector_width: metrics.border_width.max(MIN_LINK_METRIC_EXTENT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_label_metrics_project_from_host_control_metrics() {
        let mut host = current_host_metrics();
        host.font_body = 12.0;
        host.border_width = 1.5;
        host.line_height_ratio = 1.25;
        host.gap_m = 10.0;

        let metrics = axis_label_metrics_from_host(host);

        assert_eq!(metrics.font_size, 13.5);
        assert_eq!(metrics.line_height, 16.875);
        assert_eq!(metrics.link_lobe_width, 7.0);
        assert_eq!(metrics.link_lobe_height, 8.5);
        assert_eq!(metrics.link_lobe_radius, 3.5);
        assert_eq!(metrics.link_overlap, 3.0);
        assert_eq!(metrics.link_connector_width, 1.5);
    }
}
