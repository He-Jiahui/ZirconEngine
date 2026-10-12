use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::template_node_labels::template_node_label;
use super::identity::{is_icon_node, is_icon_only_node};

const LEADING_ICON_WIDTH_FRACTION: f32 = 0.28;
const LEADING_ICON_MAX_INSET_FRACTION: f32 = 0.25;
const ICON_ONLY_INSET_FRACTION: f32 = 0.16;
const MIN_IMAGE_DIMENSION: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TemplateNodeImageGeometryMetrics {
    leading_content_inset: f32,
    maximum_icon_only_inset: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn leading_icon_size(
    rect: &FrameRect,
) -> f32 {
    leading_icon_size_with_metrics(rect, template_node_image_geometry_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_rect_for_node(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    image_width: u32,
    image_height: u32,
) -> FrameRect {
    image_rect_for_node_with_metrics(
        node,
        rect,
        image_width,
        image_height,
        template_node_image_geometry_metrics(),
    )
}

fn image_rect_for_node_with_metrics(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    image_width: u32,
    image_height: u32,
    metrics: TemplateNodeImageGeometryMetrics,
) -> FrameRect {
    if !has_paintable_image_extent(rect) {
        return non_paintable_image_rect(rect);
    }
    if is_icon_node(node) {
        let label = template_node_label(node, None);
        if !label.is_empty() && !is_icon_only_node(node) {
            let inset = leading_content_inset(rect, metrics);
            let size = leading_icon_size_with_metrics(rect, metrics);
            return FrameRect {
                x: rect.x + inset,
                y: rect.y + (rect.height - size) * 0.5,
                width: size,
                height: size,
            };
        }
        let inset = (rect.width.min(rect.height) * ICON_ONLY_INSET_FRACTION)
            .min(metrics.maximum_icon_only_inset)
            .max(0.0);
        let size = (rect.width.min(rect.height) - inset * 2.0).max(MIN_IMAGE_DIMENSION);
        return FrameRect {
            x: rect.x + (rect.width - size) * 0.5,
            y: rect.y + (rect.height - size) * 0.5,
            width: size,
            height: size,
        };
    }
    fitted_image_rect(rect, image_width, image_height)
}

fn leading_icon_size_with_metrics(
    rect: &FrameRect,
    metrics: TemplateNodeImageGeometryMetrics,
) -> f32 {
    let inset = leading_content_inset(rect, metrics);
    let maximum_size = (rect.width.min(rect.height).max(0.0) - inset * 2.0).max(0.0);
    (rect.height - inset * 2.0)
        .min(rect.width * LEADING_ICON_WIDTH_FRACTION)
        .max(MIN_IMAGE_DIMENSION)
        .min(maximum_size)
}

fn leading_content_inset(rect: &FrameRect, metrics: TemplateNodeImageGeometryMetrics) -> f32 {
    metrics
        .leading_content_inset
        .min(rect.width.min(rect.height).max(0.0) * LEADING_ICON_MAX_INSET_FRACTION)
        .max(0.0)
}

fn template_node_image_geometry_metrics() -> TemplateNodeImageGeometryMetrics {
    template_node_image_geometry_metrics_from_host(current_host_metrics())
}

fn template_node_image_geometry_metrics_from_host(
    metrics: HostControlMetrics,
) -> TemplateNodeImageGeometryMetrics {
    TemplateNodeImageGeometryMetrics {
        leading_content_inset: metrics.gap_s + metrics.border_width,
        maximum_icon_only_inset: metrics.gap_s,
    }
}

fn fitted_image_rect(rect: &FrameRect, image_width: u32, image_height: u32) -> FrameRect {
    if image_width == 0 || image_height == 0 {
        return non_paintable_image_rect(rect);
    }
    let image_aspect = image_width as f32 / image_height as f32;
    let rect_aspect = rect.width / rect.height;
    if rect_aspect > image_aspect {
        let height = rect.height;
        let width = height * image_aspect;
        FrameRect {
            x: rect.x + (rect.width - width) * 0.5,
            y: rect.y,
            width,
            height,
        }
    } else {
        let width = rect.width;
        let height = width / image_aspect;
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - height) * 0.5,
            width,
            height,
        }
    }
}

fn has_paintable_image_extent(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn non_paintable_image_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.is_finite().then_some(rect.x).unwrap_or(0.0),
        y: rect.y.is_finite().then_some(rect.y).unwrap_or(0.0),
        width: 0.0,
        height: 0.0,
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
