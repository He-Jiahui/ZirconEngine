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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    context: IconButtonContext,
) -> FrameRect {
    let max_size = rect.width.min(rect.height).max(1.0);
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
