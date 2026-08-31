use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::tooltip_metrics;
use super::text::{tooltip_body, tooltip_title};
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_tooltip_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    outer: &FrameRect,
    inner: &FrameRect,
) -> bool {
    has_paintable_tooltip_extent(outer)
        && has_paintable_tooltip_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_bubble_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = tooltip_metrics();
    let width = tooltip_bubble_width(node, rect, metrics);
    FrameRect {
        x: rect.x + (rect.width - width).max(0.0) * 0.5 + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width,
        height: tooltip_bubble_height(node, metrics).min(rect.height.max(0.0)),
    }
}

fn tooltip_bubble_height(
    node: &TemplatePaneNodeData,
    metrics: super::metrics::WorkbenchTooltipMetrics,
) -> f32 {
    if tooltip_body(node).is_empty() {
        // Icon-button labels are title-only; retain only the space required to paint that line.
        metrics.title_top + metrics.title_line_height + metrics.border_width * 2.0
    } else {
        metrics.bubble_height
    }
}

fn tooltip_bubble_width(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: super::metrics::WorkbenchTooltipMetrics,
) -> f32 {
    let title_width = measure_runtime_text_width(tooltip_title(node), metrics.title_font_size);
    let body_width = measure_runtime_text_width(tooltip_body(node), metrics.body_font_size);
    let desired_width = title_width.max(body_width) + metrics.text_left * 2.0;
    let available_width = rect.width.max(0.0);
    let maximum_width = metrics
        .bubble_max_width
        .max(metrics.bubble_min_width)
        .min(available_width);
    let minimum_width = metrics.bubble_min_width.min(maximum_width);

    // Content leads the bubble width, while authored bounds remain authoritative.
    desired_width.clamp(minimum_width, maximum_width)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    rect.clone()
}

#[cfg(test)]
mod fractional_geometry_tests {
    use super::*;

    #[test]
    fn tooltip_paint_rect_preserves_fractional_post_dpi_geometry() {
        let rect = paint_rect(&FrameRect {
            x: 14.25,
            y: 19.5,
            width: 176.75,
            height: 64.25,
        });

        assert_eq!(rect.x, 14.25);
        assert_eq!(rect.y, 19.5);
        assert_eq!(rect.width, 176.75);
        assert_eq!(rect.height, 64.25);
    }
}
