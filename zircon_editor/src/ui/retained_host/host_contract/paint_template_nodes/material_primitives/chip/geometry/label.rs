use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::{
    chip_has_avatar, chip_has_icon, chip_is_deletable, chip_is_outlined, chip_is_small,
};
use super::delete::chip_delete_edge;
use super::leading::{chip_leading_edge, chip_leading_margin, chip_negative_slot_margin};
use super::metrics::{
    CHIP_LABEL_FONT_SIZE, CHIP_LABEL_OUTLINED_PADDING, CHIP_LABEL_PADDING, CHIP_LABEL_WIDTH_RATIO,
    CHIP_SMALL_LABEL_FONT_SIZE, CHIP_SMALL_LABEL_PADDING, CHIP_SMALL_OUTLINED_LABEL_PADDING,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> Option<(FrameRect, f32, f32)> {
    let font_size = chip_font_size(node);
    let line_height = font_size * 1.5;
    let left = rect.x + chip_label_left_padding(node);
    let right = rect.x + rect.width - chip_label_right_padding(node);
    if right <= left {
        return None;
    }
    let estimated_width = label.chars().count() as f32 * font_size * CHIP_LABEL_WIDTH_RATIO;
    let width = estimated_width.min(right - left).max(1.0);
    Some((
        FrameRect {
            x: left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width,
            height: line_height,
        },
        font_size,
        line_height,
    ))
}

fn chip_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else if chip_is_small(node) {
        CHIP_SMALL_LABEL_FONT_SIZE
    } else {
        CHIP_LABEL_FONT_SIZE
    }
}

fn chip_label_left_padding(node: &TemplatePaneNodeData) -> f32 {
    let base = chip_label_base_padding(node);
    if chip_has_avatar(node) || chip_has_icon(node) {
        base + chip_leading_margin(node) + chip_leading_edge(node) - chip_negative_slot_margin(node)
    } else {
        base
    }
}

fn chip_label_right_padding(node: &TemplatePaneNodeData) -> f32 {
    let base = chip_label_base_padding(node);
    if chip_is_deletable(node) {
        base + chip_delete_edge(node) - chip_negative_slot_margin(node)
    } else {
        base
    }
}

fn chip_label_base_padding(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        if chip_is_outlined(node) {
            CHIP_SMALL_OUTLINED_LABEL_PADDING
        } else {
            CHIP_SMALL_LABEL_PADDING
        }
    } else if chip_is_outlined(node) {
        CHIP_LABEL_OUTLINED_PADDING
    } else {
        CHIP_LABEL_PADDING
    }
}
