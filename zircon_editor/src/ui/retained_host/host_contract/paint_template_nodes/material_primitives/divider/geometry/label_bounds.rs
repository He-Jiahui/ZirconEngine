use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::component_variant_contains;
use super::align::pixel_aligned;
use super::metrics::{
    divider_font_size, DIVIDER_WRAPPER_HORIZONTAL_PADDING, DIVIDER_WRAPPER_VERTICAL_PADDING,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_label_bounds(
    node: &TemplatePaneNodeData,
    line_start: f32,
    line_end: f32,
    label: &str,
) -> (f32, f32) {
    let label_width = estimated_horizontal_label_width(node, label, line_end - line_start);
    let label_left = horizontal_label_left(node, line_start, line_end, label_width);
    (label_left, (label_left + label_width).min(line_end))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vertical_label_bounds(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    line_bottom: f32,
) -> (f32, f32) {
    let label_height = estimated_vertical_label_height(node, rect);
    let label_top = pixel_aligned(rect.y + (rect.height - label_height).max(0.0) * 0.5);
    (label_top, (label_top + label_height).min(line_bottom))
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

enum DividerTextAlign {
    Left,
    Center,
    Right,
}
