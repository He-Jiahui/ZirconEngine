use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::{chip_has_avatar, chip_has_icon, chip_is_deletable, chip_is_outlined, chip_is_small};

const CHIP_MEDIUM_HEIGHT: f32 = 32.0;
const CHIP_SMALL_HEIGHT: f32 = 24.0;
const CHIP_LABEL_FONT_SIZE: f32 = 13.0;
const CHIP_SMALL_LABEL_FONT_SIZE: f32 = 12.0;
const CHIP_LABEL_WIDTH_RATIO: f32 = 0.56;
const CHIP_LABEL_PADDING: f32 = 12.0;
const CHIP_LABEL_OUTLINED_PADDING: f32 = 11.0;
const CHIP_SMALL_LABEL_PADDING: f32 = 8.0;
const CHIP_SMALL_OUTLINED_LABEL_PADDING: f32 = 7.0;
const CHIP_AVATAR_MEDIUM_EDGE: f32 = 24.0;
const CHIP_AVATAR_SMALL_EDGE: f32 = 18.0;
const CHIP_ICON_MEDIUM_EDGE: f32 = 20.0;
const CHIP_ICON_SMALL_EDGE: f32 = 18.0;
const CHIP_DELETE_MEDIUM_EDGE: f32 = 22.0;
const CHIP_DELETE_SMALL_EDGE: f32 = 16.0;

pub(super) fn chip_frame(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let target_height = chip_height(node).min(rect.height.max(1.0)).round();
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - target_height).max(0.0) * 0.5).round(),
        width: rect.width.round().max(1.0),
        height: target_height.max(1.0),
    }
}

pub(super) fn chip_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured > 0.0 {
        configured.min(rect.height * 0.5)
    } else {
        rect.height * 0.5
    }
}

pub(super) fn chip_avatar_frame(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    chip_leading_slot_frame(node, rect, chip_avatar_edge(node, rect))
}

pub(super) fn chip_icon_frame(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    chip_leading_slot_frame(node, rect, chip_icon_edge(node, rect))
}

pub(super) fn chip_label_frame(
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

pub(super) fn chip_delete_icon_frame(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let edge = chip_delete_icon_edge(node, rect);
    let right_margin = if chip_is_small(node) { 4.0 } else { 5.0 };
    FrameRect {
        x: rect.x + rect.width - right_margin - edge,
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    }
}

fn chip_height(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_SMALL_HEIGHT
    } else {
        CHIP_MEDIUM_HEIGHT
    }
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

fn chip_leading_slot_frame(node: &TemplatePaneNodeData, rect: &FrameRect, edge: f32) -> FrameRect {
    FrameRect {
        x: rect.x + chip_leading_margin(node),
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    }
}

fn chip_leading_edge(node: &TemplatePaneNodeData) -> f32 {
    if chip_has_avatar(node) {
        if chip_is_small(node) {
            CHIP_AVATAR_SMALL_EDGE
        } else {
            CHIP_AVATAR_MEDIUM_EDGE
        }
    } else if chip_has_icon(node) {
        if chip_is_small(node) {
            CHIP_ICON_SMALL_EDGE
        } else {
            CHIP_ICON_MEDIUM_EDGE
        }
    } else {
        0.0
    }
}

fn chip_avatar_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if chip_is_small(node) {
        CHIP_AVATAR_SMALL_EDGE
    } else {
        CHIP_AVATAR_MEDIUM_EDGE
    }
    .min(rect.height - 4.0)
    .max(1.0)
}

fn chip_icon_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if chip_is_small(node) {
        CHIP_ICON_SMALL_EDGE
    } else {
        CHIP_ICON_MEDIUM_EDGE
    }
    .min(rect.height - 4.0)
    .max(1.0)
}

fn chip_delete_edge(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_DELETE_SMALL_EDGE
    } else {
        CHIP_DELETE_MEDIUM_EDGE
    }
}

fn chip_delete_icon_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    chip_delete_edge(node).min(rect.height - 4.0).max(1.0)
}

fn chip_leading_margin(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        if chip_is_outlined(node) {
            2.0
        } else {
            4.0
        }
    } else if chip_is_outlined(node) {
        4.0
    } else {
        5.0
    }
}

fn chip_negative_slot_margin(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        4.0
    } else {
        6.0
    }
}
