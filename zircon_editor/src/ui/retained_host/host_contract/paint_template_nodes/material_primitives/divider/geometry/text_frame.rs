use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{
    divider_font_size, DIVIDER_WRAPPER_HORIZONTAL_PADDING, DIVIDER_WRAPPER_VERTICAL_PADDING,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_label_text_frame(
    node: &TemplatePaneNodeData,
    label: &str,
    label_left: f32,
    label_right: f32,
    rect: &FrameRect,
) -> Option<(FrameRect, f32, f32)> {
    if label.trim().is_empty() || label_right <= label_left {
        return None;
    }
    let font_size = divider_font_size(node, rect.height);
    let line_height = font_size * 1.2;
    let text_left = label_left + DIVIDER_WRAPPER_HORIZONTAL_PADDING;
    let text_right = (label_right - DIVIDER_WRAPPER_HORIZONTAL_PADDING).max(text_left);
    Some((
        FrameRect {
            x: text_left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_right - text_left,
            height: line_height,
        },
        font_size,
        line_height,
    ))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vertical_label_text_frame(
    node: &TemplatePaneNodeData,
    label: &str,
    label_top: f32,
    label_bottom: f32,
    rect: &FrameRect,
) -> Option<(FrameRect, f32, f32)> {
    if label.trim().is_empty() || label_bottom <= label_top {
        return None;
    }
    let font_size = divider_font_size(node, rect.height);
    let line_height = font_size * 1.2;
    let horizontal_padding = DIVIDER_WRAPPER_HORIZONTAL_PADDING.min(rect.width * 0.25);
    let text_top = label_top + DIVIDER_WRAPPER_VERTICAL_PADDING;
    Some((
        FrameRect {
            x: rect.x + horizontal_padding,
            y: text_top,
            width: (rect.width - horizontal_padding * 2.0).max(1.0),
            height: (label_bottom - text_top).min(line_height).max(1.0),
        },
        font_size,
        line_height,
    ))
}
