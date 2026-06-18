use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::component_variant_contains;

const BADGE_STANDARD_HEIGHT: f32 = 20.0;
const BADGE_STANDARD_MIN_WIDTH: f32 = 20.0;
const BADGE_STANDARD_PADDING_X: f32 = 6.0;
const BADGE_DOT_EDGE: f32 = 8.0;
const BADGE_STANDARD_RADIUS: f32 = 10.0;
const BADGE_DOT_RADIUS: f32 = 4.0;
const BADGE_FONT_SIZE: f32 = 12.0;
const BADGE_ROOT_FONT_SIZE: f32 = 12.0;
const BADGE_ROOT_TEXT_INSET_X: f32 = 8.0;
const BADGE_ROOT_TEXT_WIDTH_RATIO: f32 = 0.56;
const BADGE_CIRCULAR_OFFSET_RATIO: f32 = 0.14;
const BADGE_TEXT_WIDTH_RATIO: f32 = 0.56;

pub(super) struct BadgeTextFrame {
    pub(super) rect: FrameRect,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
}

pub(super) fn badge_root_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> BadgeTextFrame {
    let font_size = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        BADGE_ROOT_FONT_SIZE
    };
    let line_height = font_size * 1.2;
    let available_width = (rect.width - BADGE_ROOT_TEXT_INSET_X * 2.0).max(1.0);
    let text_width = (label.chars().count() as f32 * font_size * BADGE_ROOT_TEXT_WIDTH_RATIO)
        .min(available_width)
        .max(1.0);
    BadgeTextFrame {
        rect: FrameRect {
            x: rect.x + BADGE_ROOT_TEXT_INSET_X,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    }
}

pub(super) fn badge_overlay_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    display: &str,
    dot: bool,
) -> FrameRect {
    let (width, height) = if dot {
        (BADGE_DOT_EDGE, BADGE_DOT_EDGE)
    } else {
        (
            (display.chars().count() as f32 * BADGE_FONT_SIZE * BADGE_TEXT_WIDTH_RATIO
                + BADGE_STANDARD_PADDING_X * 2.0)
                .max(BADGE_STANDARD_MIN_WIDTH),
            BADGE_STANDARD_HEIGHT,
        )
    };
    let (anchor_x, anchor_y) = badge_anchor_point(node, rect);
    FrameRect {
        x: (anchor_x - width * 0.5).round(),
        y: (anchor_y - height * 0.5).round(),
        width: width.round().max(1.0),
        height: height.round().max(1.0),
    }
}

pub(super) fn badge_overlay_radius(dot: bool) -> f32 {
    if dot {
        BADGE_DOT_RADIUS
    } else {
        BADGE_STANDARD_RADIUS
    }
}

pub(super) fn badge_overlay_text_frame(display: &str, rect: &FrameRect) -> BadgeTextFrame {
    let line_height = BADGE_FONT_SIZE;
    let text_width = (display.chars().count() as f32 * BADGE_FONT_SIZE * BADGE_TEXT_WIDTH_RATIO)
        .min(rect.width)
        .max(1.0);
    BadgeTextFrame {
        rect: FrameRect {
            x: rect.x + (rect.width - text_width).max(0.0) * 0.5,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        font_size: BADGE_FONT_SIZE,
        line_height,
    }
}

fn badge_anchor_point(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let circular = component_variant_contains(node, "circular")
        || component_variant_contains(node, "overlapCircular");
    let offset_x = if circular {
        rect.width * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let offset_y = if circular {
        rect.height * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let left = badge_is_left_anchored(node);
    let bottom = badge_is_bottom_anchored(node);
    let x = if left {
        rect.x + offset_x
    } else {
        rect.x + rect.width - offset_x
    };
    let y = if bottom {
        rect.y + rect.height - offset_y
    } else {
        rect.y + offset_y
    };
    (x, y)
}

fn badge_is_left_anchored(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "left")
        || component_variant_contains(node, "anchorOriginTopLeft")
        || component_variant_contains(node, "anchorOriginBottomLeft")
}

fn badge_is_bottom_anchored(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "bottom")
        || component_variant_contains(node, "anchorOriginBottomLeft")
        || component_variant_contains(node, "anchorOriginBottomRight")
}
