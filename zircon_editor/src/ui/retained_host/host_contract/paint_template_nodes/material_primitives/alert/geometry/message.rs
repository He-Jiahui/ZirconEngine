use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::identity::alert_has_icon;
use super::action::alert_action_width;
use super::metrics::{
    ALERT_FONT_SIZE, ALERT_ICON_EDGE, ALERT_ICON_GAP, ALERT_PADDING_X, ALERT_TEXT_WIDTH_RATIO,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_message_left(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    if alert_has_icon(node) {
        rect.x + ALERT_PADDING_X + ALERT_ICON_EDGE + ALERT_ICON_GAP
    } else {
        rect.x + ALERT_PADDING_X
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_message_right(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    rect.x + rect.width - ALERT_PADDING_X - alert_action_width(node)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_message_frame(
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

fn alert_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        ALERT_FONT_SIZE
    }
}
