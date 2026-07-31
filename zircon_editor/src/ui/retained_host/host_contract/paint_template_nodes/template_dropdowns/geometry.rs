use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_dropdown_metrics::WorkbenchDropdownMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    rect
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round(),
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_dropdown_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
        && (rect.x + rect.width).is_finite()
        && (rect.y + rect.height).is_finite()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    outer: &FrameRect,
    inner: &FrameRect,
) -> bool {
    has_paintable_dropdown_extent(outer)
        && has_paintable_dropdown_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron_fits(
    rect: &FrameRect,
    metrics: &WorkbenchDropdownMetrics,
) -> bool {
    metrics.chevron_size.is_finite()
        && metrics.chevron_size > 0.0
        && metrics.chevron_right.is_finite()
        && metrics.chevron_right >= 0.0
        && rect.width >= metrics.chevron_right + metrics.chevron_size
        && rect.height >= metrics.chevron_size
}
