use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::search::search_field_paint_rect;
use crate::ui::retained_host::host_contract::paint_geometry::{
    corner_radius_for_frame, inward_pixel_aligned_rect,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    search_field_paint_rect(node, rect)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_field_extent(
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
    inner: &FrameRect,
    outer: &FrameRect,
) -> bool {
    if !has_paintable_field_extent(inner) || !has_paintable_field_extent(outer) {
        return false;
    }

    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    inner_right.is_finite()
        && inner_bottom.is_finite()
        && outer_right.is_finite()
        && outer_bottom.is_finite()
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    inward_pixel_aligned_rect(rect)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_surface_radius(
    rect: &FrameRect,
    requested_radius: f32,
) -> f32 {
    corner_radius_for_frame(rect, requested_radius)
}
