use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::component_variant_contains;

const DIVIDER_THICKNESS: f32 = 1.0;
const DIVIDER_MIDDLE_HORIZONTAL_INSET: f32 = 16.0;
const DIVIDER_INSET_HORIZONTAL_INSET: f32 = 72.0;
const DIVIDER_MIDDLE_VERTICAL_INSET: f32 = 8.0;
const DIVIDER_WRAPPER_HORIZONTAL_PADDING: f32 = 9.6;
const DIVIDER_WRAPPER_VERTICAL_PADDING: f32 = 9.6;
const DIVIDER_DEFAULT_FONT_SIZE: f32 = 12.0;
const DIVIDER_MIN_FONT_SIZE: f32 = 8.0;

pub(super) fn divider_is_vertical(node: &TemplatePaneNodeData, rect: &FrameRect) -> bool {
    component_variant_contains(node, "vertical")
        || component_variant_contains(node, "wrapperVertical")
        || (!component_variant_contains(node, "horizontal") && rect.height > rect.width * 1.4)
}

pub(super) fn horizontal_divider_extent(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> (f32, f32) {
    let mut start = rect.x;
    let mut end = rect.x + rect.width;
    if component_variant_contains(node, "middle") {
        let inset = DIVIDER_MIDDLE_HORIZONTAL_INSET.min(rect.width * 0.45);
        start += inset;
        end -= inset;
    } else if component_variant_contains(node, "inset") {
        let inset = DIVIDER_INSET_HORIZONTAL_INSET.min(rect.width * 0.9);
        start += inset;
    }
    if end < start {
        let center = rect.x + rect.width * 0.5;
        (center, center)
    } else {
        (pixel_aligned(start), pixel_aligned(end))
    }
}

pub(super) fn vertical_divider_extent(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let mut top = rect.y;
    let mut bottom = rect.y + rect.height;
    if component_variant_contains(node, "middle") {
        let inset = DIVIDER_MIDDLE_VERTICAL_INSET.min(rect.height * 0.45);
        top += inset;
        bottom -= inset;
    }
    if bottom < top {
        let center = rect.y + rect.height * 0.5;
        (center, center)
    } else {
        (pixel_aligned(top), pixel_aligned(bottom))
    }
}

pub(super) fn horizontal_line_y(rect: &FrameRect) -> f32 {
    pixel_aligned(rect.y + (rect.height - DIVIDER_THICKNESS).max(0.0) * 0.5)
}

pub(super) fn vertical_line_x(rect: &FrameRect) -> f32 {
    pixel_aligned(rect.x + (rect.width - DIVIDER_THICKNESS).max(0.0) * 0.5)
}

pub(super) fn horizontal_line_frame(left: f32, right: f32, y: f32) -> Option<FrameRect> {
    let width = right - left;
    (width > 0.5).then(|| FrameRect {
        x: left,
        y,
        width,
        height: DIVIDER_THICKNESS,
    })
}

pub(super) fn vertical_line_frame(x: f32, top: f32, bottom: f32) -> Option<FrameRect> {
    let height = bottom - top;
    (height > 0.5).then(|| FrameRect {
        x,
        y: top,
        width: DIVIDER_THICKNESS,
        height,
    })
}

pub(super) fn horizontal_label_bounds(
    node: &TemplatePaneNodeData,
    line_start: f32,
    line_end: f32,
    label: &str,
) -> (f32, f32) {
    let label_width = estimated_horizontal_label_width(node, label, line_end - line_start);
    let label_left = horizontal_label_left(node, line_start, line_end, label_width);
    (label_left, (label_left + label_width).min(line_end))
}

pub(super) fn vertical_label_bounds(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    line_bottom: f32,
) -> (f32, f32) {
    let label_height = estimated_vertical_label_height(node, rect);
    let label_top = pixel_aligned(rect.y + (rect.height - label_height).max(0.0) * 0.5);
    (label_top, (label_top + label_height).min(line_bottom))
}

pub(super) fn horizontal_label_text_frame(
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

pub(super) fn vertical_label_text_frame(
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

fn horizontal_label_left(
    node: &TemplatePaneNodeData,
    line_start: f32,
    line_end: f32,
    label_width: f32,
) -> f32 {
    let available = (line_end - line_start).max(0.0);
    let remaining = (available - label_width).max(0.0);
    let ratio = match divider_text_align(node) {
        DividerTextAlign::Left => 0.1,
        DividerTextAlign::Center => 0.5,
        DividerTextAlign::Right => 0.9,
    };
    pixel_aligned(line_start + remaining * ratio)
}

fn estimated_horizontal_label_width(
    node: &TemplatePaneNodeData,
    label: &str,
    available_width: f32,
) -> f32 {
    let font_size = divider_font_size(node, available_width);
    let text_width = label.chars().count() as f32 * font_size * 0.56;
    (text_width + DIVIDER_WRAPPER_HORIZONTAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_HORIZONTAL_PADDING * 2.0)
        .min(available_width.max(0.0))
}

fn estimated_vertical_label_height(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let font_size = divider_font_size(node, rect.height);
    (font_size * 1.2 + DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .min(rect.height.max(0.0))
}

fn divider_text_align(node: &TemplatePaneNodeData) -> DividerTextAlign {
    if component_variant_contains(node, "textAlignRight")
        || component_variant_contains(node, "right")
        || matches!(node.text_align.as_str(), "right" | "end")
    {
        DividerTextAlign::Right
    } else if component_variant_contains(node, "textAlignLeft")
        || component_variant_contains(node, "left")
        || matches!(node.text_align.as_str(), "left" | "start")
    {
        DividerTextAlign::Left
    } else {
        DividerTextAlign::Center
    }
}

fn divider_font_size(node: &TemplatePaneNodeData, available_height: f32) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DIVIDER_DEFAULT_FONT_SIZE
    };
    requested
        .min((available_height * 0.82).max(DIVIDER_MIN_FONT_SIZE))
        .max(DIVIDER_MIN_FONT_SIZE)
}

fn pixel_aligned(value: f32) -> f32 {
    value.round()
}

enum DividerTextAlign {
    Left,
    Center,
    Right,
}
