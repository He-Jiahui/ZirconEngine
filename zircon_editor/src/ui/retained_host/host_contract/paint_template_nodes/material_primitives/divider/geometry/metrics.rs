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

// 分隔线厚度、内缩和标签容器间距统一来自宿主度量，避免与工作台密度配置分叉。
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
#[path = "tests/metrics.rs"]
mod tests;
