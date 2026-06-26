use super::super::super::data::{FrameRect, TemplatePaneNodeData};
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
    let default_size = match context {
        IconButtonContext::Rail => (max_size * 0.48).clamp(18.0, 24.0),
        IconButtonContext::Toolbar => (max_size * 0.67).clamp(18.0, 20.0),
        IconButtonContext::Panel => (max_size * 0.50).clamp(15.0, 21.0),
    };
    let size = if node.value_number.is_finite() && node.value_number > 0.0 {
        node.value_number
    } else {
        default_size
    }
    .min((max_size - 6.0).max(1.0));
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}
