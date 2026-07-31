use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{label_text_slot_width, max_label_slot_width};

const CENTER_FACTOR: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ButtonContentLayout {
    pub(super) text_slot_width: f32,
    pub(super) start_x: f32,
}

pub(super) fn button_content_layout(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    glyph_width: f32,
    chevron_width: f32,
    label_ink_width: f32,
) -> ButtonContentLayout {
    let max_content_width = max_label_slot_width(node, rect);
    let max_text_width = (max_content_width - glyph_width - chevron_width).max(0.0);
    let visual_label_width = label_ink_width.max(0.0).min(max_text_width);
    let text_slot_width = label_text_slot_width(visual_label_width, max_text_width);
    let content_width = (visual_label_width + glyph_width + chevron_width).min(max_content_width);
    ButtonContentLayout {
        text_slot_width,
        start_x: rect.x + centered_offset(rect.width, content_width),
    }
}

pub(super) fn content_centered_y(rect: &FrameRect, content_height: f32) -> f32 {
    rect.y + centered_offset(rect.height, content_height)
}

fn centered_offset(container_extent: f32, content_extent: f32) -> f32 {
    (container_extent - content_extent).max(0.0) * CENTER_FACTOR
}
