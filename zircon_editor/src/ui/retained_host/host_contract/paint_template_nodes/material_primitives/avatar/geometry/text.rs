use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::metrics::{AVATAR_MIN_FONT_SIZE, AVATAR_TEXT_FONT_RATIO, AVATAR_TEXT_WIDTH_RATIO};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> (FrameRect, f32, f32) {
    let font_size = avatar_font_size(node, rect);
    let line_height = font_size;
    let text_width = (label.chars().count() as f32 * font_size * AVATAR_TEXT_WIDTH_RATIO)
        .min(rect.width)
        .max(1.0);
    (
        FrameRect {
            x: rect.x + (rect.width - text_width).max(0.0) * 0.5,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    )
}

fn avatar_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        rect.width.min(rect.height) * AVATAR_TEXT_FONT_RATIO
    };
    requested.max(AVATAR_MIN_FONT_SIZE).min(rect.height)
}
