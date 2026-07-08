use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::workbench_field_metrics;
use super::search::search_field_paint_rect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    search_field_paint_rect(node, rect)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    let min_extent = workbench_field_metrics().min_paint_rect_extent;
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(min_extent),
        height: rect.height.max(min_extent),
    }
}
