use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

const DIVIDER_MIDDLE_HORIZONTAL_GAP_FACTOR: f32 = 2.0;
const DIVIDER_INSET_HORIZONTAL_GAP_FACTOR: f32 = 9.0;
const DIVIDER_MAX_FONT_HEIGHT_RATIO: f32 = 0.82;
const DIVIDER_LABEL_CENTER_RATIO: f32 = 0.5;
const DIVIDER_VERTICAL_TEXT_HORIZONTAL_PADDING_RATIO: f32 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
struct DividerGeometryMetrics {
    thickness: f32,
    middle_horizontal_inset: f32,
    inset_horizontal_inset: f32,
    middle_vertical_inset: f32,
    wrapper_horizontal_padding: f32,
    wrapper_vertical_padding: f32,
    default_font_size: f32,
    minimum_font_size: f32,
    line_height_ratio: f32,
    minimum_text_frame_extent: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_thickness() -> f32
{
    divider_geometry_metrics().thickness
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_middle_horizontal_inset(
) -> f32 {
    divider_geometry_metrics().middle_horizontal_inset
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_inset_horizontal_inset(
) -> f32 {
    divider_geometry_metrics().inset_horizontal_inset
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_middle_vertical_inset(
) -> f32 {
    divider_geometry_metrics().middle_vertical_inset
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_wrapper_horizontal_padding(
) -> f32 {
    divider_geometry_metrics().wrapper_horizontal_padding
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_wrapper_vertical_padding(
) -> f32 {
    divider_geometry_metrics().wrapper_vertical_padding
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_font_size(
    node: &TemplatePaneNodeData,
    available_height: f32,
) -> f32 {
    divider_font_size_from_metrics(node, available_height, divider_geometry_metrics())
}

fn divider_font_size_from_metrics(
    node: &TemplatePaneNodeData,
    available_height: f32,
    metrics: DividerGeometryMetrics,
) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        metrics.default_font_size
    };
    requested
        .min((available_height * DIVIDER_MAX_FONT_HEIGHT_RATIO).max(metrics.minimum_font_size))
        .max(metrics.minimum_font_size)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_label_line_height(
    font_size: f32,
) -> f32 {
    font_size * divider_geometry_metrics().line_height_ratio
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_wrapped_label_width(
    measured_text_width: f32,
    available_width: f32,
) -> f32 {
    let padding = divider_wrapper_horizontal_padding();
    (measured_text_width + padding * 2.0)
        .max(padding * 2.0)
        .min(available_width.max(0.0))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_centered_label_y(
    rect: &FrameRect,
    line_height: f32,
) -> f32 {
    rect.y + (rect.height - line_height).max(0.0) * DIVIDER_LABEL_CENTER_RATIO
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_vertical_label_height(
    font_size: f32,
    rect_height: f32,
) -> f32 {
    let padding = divider_wrapper_vertical_padding();
    (divider_label_line_height(font_size) + padding * 2.0)
        .max(padding * 2.0)
        .min(rect_height.max(0.0))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_vertical_text_horizontal_padding(
    rect_width: f32,
) -> f32 {
    divider_wrapper_horizontal_padding()
        .min(rect_width * DIVIDER_VERTICAL_TEXT_HORIZONTAL_PADDING_RATIO)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_min_text_frame_extent(
    extent: f32,
) -> f32 {
    extent.max(divider_geometry_metrics().minimum_text_frame_extent)
}

fn divider_geometry_metrics() -> DividerGeometryMetrics {
    divider_geometry_metrics_from_host(current_host_metrics())
}

fn divider_geometry_metrics_from_host(metrics: HostControlMetrics) -> DividerGeometryMetrics {
    let wrapper_padding = metrics.gap_m + metrics.border_width;
    DividerGeometryMetrics {
        thickness: metrics.border_width,
        middle_horizontal_inset: metrics.gap_m * DIVIDER_MIDDLE_HORIZONTAL_GAP_FACTOR,
        inset_horizontal_inset: metrics.gap_m * DIVIDER_INSET_HORIZONTAL_GAP_FACTOR,
        middle_vertical_inset: metrics.gap_m,
        wrapper_horizontal_padding: wrapper_padding,
        wrapper_vertical_padding: wrapper_padding,
        default_font_size: metrics.font_body,
        minimum_font_size: metrics.font_small,
        line_height_ratio: metrics.line_height_ratio,
        minimum_text_frame_extent: metrics.border_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn divider_text_metrics_project_wrapper_and_centering_rules() {
        let rect = FrameRect {
            x: 0.0,
            y: 10.0,
            width: 160.0,
            height: 40.0,
        };

        let line_height = divider_label_line_height(12.0);
        let wrapped_width = divider_wrapped_label_width(32.0, 120.0);
        let metrics = divider_geometry_metrics_from_host(METRICS);

        assert!((line_height - METRICS.line_height(12.0)).abs() <= 0.01);
        assert!((wrapped_width - (32.0 + metrics.wrapper_horizontal_padding * 2.0)).abs() <= 0.01);
        assert!((divider_centered_label_y(&rect, line_height) - 22.8).abs() <= 0.01);
    }

    #[test]
    fn divider_vertical_text_metrics_clamp_padding_and_min_extent() {
        assert!((divider_vertical_text_horizontal_padding(24.0) - 6.0).abs() <= 0.01);
        assert!((divider_min_text_frame_extent(0.2) - 1.0).abs() <= 0.01);
        assert!((divider_vertical_label_height(12.0, 28.0) - 28.0).abs() <= 0.01);
    }

    #[test]
    fn divider_geometry_metrics_project_from_shared_host_metrics() {
        let host = HostControlMetrics {
            border_width: 1.5,
            font_small: 10.0,
            font_body: 15.0,
            line_height_ratio: 1.4,
            gap_m: 7.0,
            ..METRICS
        };

        let metrics = divider_geometry_metrics_from_host(host);

        assert_eq!(metrics.thickness, 1.5);
        assert_eq!(metrics.middle_horizontal_inset, 14.0);
        assert_eq!(metrics.inset_horizontal_inset, 63.0);
        assert_eq!(metrics.middle_vertical_inset, 7.0);
        assert_eq!(metrics.wrapper_horizontal_padding, 8.5);
        assert_eq!(metrics.wrapper_vertical_padding, 8.5);
        assert_eq!(metrics.default_font_size, 15.0);
        assert_eq!(metrics.minimum_font_size, 10.0);
        assert_eq!(metrics.line_height_ratio, 1.4);
        assert_eq!(metrics.minimum_text_frame_extent, 1.5);

        let authored = TemplatePaneNodeData {
            font_size: 17.5,
            ..TemplatePaneNodeData::default()
        };
        assert_eq!(
            divider_font_size_from_metrics(&TemplatePaneNodeData::default(), 40.0, metrics),
            15.0
        );
        assert_eq!(
            divider_font_size_from_metrics(&authored, 40.0, metrics),
            17.5
        );
    }
}
