use super::super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::align::pixel_aligned;
use super::super::metrics::{divider_font_size, DIVIDER_WRAPPER_VERTICAL_PADDING};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vertical_label_bounds(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    line_bottom: f32,
) -> (f32, f32) {
    let label_height = estimated_vertical_label_height(node, rect);
    let label_top = pixel_aligned(rect.y + (rect.height - label_height).max(0.0) * 0.5);
    (label_top, (label_top + label_height).min(line_bottom))
}

fn estimated_vertical_label_height(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let font_size = divider_font_size(node, rect.height);
    (font_size * 1.2 + DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .min(rect.height.max(0.0))
}
