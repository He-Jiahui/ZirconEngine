use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::tooltip_metrics;

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
    FrameRect {
        x: rect.x + (rect.width - metrics.bubble_width).max(0.0) * 0.5 + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: metrics.bubble_width.min(rect.width.max(0.0)),
        height: metrics.bubble_height.min(rect.height.max(0.0)),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    let x = rect.x.ceil();
    let y = rect.y.ceil();
    FrameRect {
        x,
        y,
        width: ((rect.x + rect.width).floor() - x).max(0.0),
        height: ((rect.y + rect.height).floor() - y).max(0.0),
    }
}
