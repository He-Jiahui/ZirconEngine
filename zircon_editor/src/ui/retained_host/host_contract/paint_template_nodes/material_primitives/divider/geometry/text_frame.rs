use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{
    divider_centered_label_y, divider_font_size, divider_label_line_height,
    divider_min_text_frame_extent, divider_vertical_text_horizontal_padding,
    DIVIDER_WRAPPER_HORIZONTAL_PADDING, DIVIDER_WRAPPER_VERTICAL_PADDING,
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
    let line_height = divider_label_line_height(font_size);
    let text_left = label_left + DIVIDER_WRAPPER_HORIZONTAL_PADDING;
    let text_right = (label_right - DIVIDER_WRAPPER_HORIZONTAL_PADDING).max(text_left);
    Some((
        FrameRect {
            x: text_left,
            y: divider_centered_label_y(rect, line_height),
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
    let line_height = divider_label_line_height(font_size);
    let horizontal_padding = divider_vertical_text_horizontal_padding(rect.width);
    let text_top = label_top + DIVIDER_WRAPPER_VERTICAL_PADDING;
    Some((
        FrameRect {
            x: rect.x + horizontal_padding,
            y: text_top,
            width: divider_min_text_frame_extent(rect.width - horizontal_padding * 2.0),
            height: divider_min_text_frame_extent((label_bottom - text_top).min(line_height)),
        },
        font_size,
        line_height,
    ))
}
