use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::{alert_has_action, alert_has_icon};

const ALERT_PADDING_X: f32 = 16.0;
const ALERT_ICON_EDGE: f32 = 22.0;
const ALERT_ICON_GAP: f32 = 12.0;
const ALERT_ACTION_EDGE: f32 = 20.0;
const ALERT_ACTION_GAP: f32 = 16.0;
const ALERT_ACTION_TRAILING: f32 = 8.0;
const ALERT_FONT_SIZE: f32 = 13.0;
const ALERT_TEXT_WIDTH_RATIO: f32 = 0.56;
const ALERT_ICON_MARK_EDGE: f32 = 14.0;

pub(super) fn alert_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

pub(super) fn alert_message_left(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if alert_has_icon(node) {
        rect.x + ALERT_PADDING_X + ALERT_ICON_EDGE + ALERT_ICON_GAP
    } else {
        rect.x + ALERT_PADDING_X
    }
}

pub(super) fn alert_message_right(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    rect.x + rect.width - ALERT_PADDING_X - alert_action_width(node)
}

pub(super) fn alert_action_width(node: &TemplatePaneNodeData) -> f32 {
    if alert_has_action(node) {
        ALERT_ACTION_EDGE + ALERT_ACTION_GAP
    } else {
        0.0
    }
}

pub(super) fn alert_icon_frame(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + ALERT_PADDING_X,
        y: rect.y + (rect.height - ALERT_ICON_EDGE).max(0.0) * 0.5,
        width: ALERT_ICON_EDGE,
        height: ALERT_ICON_EDGE,
    }
}

pub(super) fn alert_icon_mark_frame(frame: &FrameRect) -> FrameRect {
    FrameRect {
        x: frame.x + (frame.width - ALERT_ICON_MARK_EDGE) * 0.5,
        y: frame.y + (frame.height - ALERT_ICON_MARK_EDGE) * 0.5,
        width: ALERT_ICON_MARK_EDGE,
        height: ALERT_ICON_MARK_EDGE,
    }
}

pub(super) fn alert_message_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    left: f32,
    right: f32,
    message: &str,
) -> Option<(FrameRect, f32, f32)> {
    if right <= left {
        return None;
    }
    let font_size = alert_font_size(node);
    let line_height = font_size * 1.45;
    let width = (message.chars().count() as f32 * font_size * ALERT_TEXT_WIDTH_RATIO)
        .min(right - left)
        .max(1.0);
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

pub(super) fn alert_action_frame(rect: &FrameRect) -> FrameRect {
    let edge = ALERT_ACTION_EDGE.min(rect.height - 8.0).max(1.0);
    FrameRect {
        x: rect.x + rect.width - ALERT_ACTION_TRAILING - edge,
        y: rect.y + (rect.height - edge).max(0.0) * 0.5,
        width: edge,
        height: edge,
    }
}

fn alert_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        ALERT_FONT_SIZE
    }
}
