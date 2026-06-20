use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::anchor::badge_anchor_point;
use super::metrics::{
    BADGE_DOT_EDGE, BADGE_DOT_RADIUS, BADGE_FONT_SIZE, BADGE_STANDARD_HEIGHT,
    BADGE_STANDARD_MIN_WIDTH, BADGE_STANDARD_PADDING_X, BADGE_STANDARD_RADIUS,
    BADGE_TEXT_WIDTH_RATIO,
};
use super::model::BadgeTextFrame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_frame(
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_radius(
    dot: bool,
) -> f32 {
    if dot {
        BADGE_DOT_RADIUS
    } else {
        BADGE_STANDARD_RADIUS
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_text_frame(
    display: &str,
    rect: &FrameRect,
) -> BadgeTextFrame {
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
