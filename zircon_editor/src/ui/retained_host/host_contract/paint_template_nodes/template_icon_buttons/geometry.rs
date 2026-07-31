use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::icon_button_glyph_metrics;
use super::style::IconButtonContext;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_icon_button_extent(
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
    if !has_paintable_icon_button_extent(inner) || !has_paintable_icon_button_extent(outer) {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    context: IconButtonContext,
) -> FrameRect {
    let max_size = rect.width.min(rect.height).max(0.0);
    let metrics = icon_button_glyph_metrics();
    let default_size = metrics.glyph_size_for_context(context);
    let size = if node.value_number.is_finite() && node.value_number > 0.0 {
        node.value_number
    } else {
        default_size
    }
    .min(metrics.max_glyph_size(max_size));
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_is_paintable(
    glyph: &FrameRect,
    button: &FrameRect,
    context: IconButtonContext,
) -> bool {
    let required_size = icon_button_glyph_metrics().glyph_size_for_context(context);
    frame_is_within(glyph, button) && glyph.width >= required_size && glyph.height >= required_size
}
