//! 将宿主字体、边框与间距映射为变换轴标签和链接图标密度；默认数值保留既有视觉基线。
//! 主题变更后的绘制应重新取得快照，不应把测得值当成静态常量缓存。

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
#[path = "tests/projection.rs"]
mod tests;
