pub(super) use super::super::super::bounded_extent as alert_bounded_extent;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(super) const ALERT_PADDING_X: f32 = 16.0;
pub(super) const ALERT_ICON_EDGE: f32 = 22.0;
pub(super) const ALERT_ICON_GAP: f32 = 12.0;
pub(super) const ALERT_ACTION_EDGE: f32 = 20.0;
pub(super) const ALERT_ACTION_GAP: f32 = 16.0;
pub(super) const ALERT_ACTION_TRAILING: f32 = 8.0;
pub(super) const ALERT_FONT_SIZE: f32 = 13.0;
pub(super) const ALERT_ICON_MARK_EDGE: f32 = 14.0;

const ALERT_MESSAGE_LINE_HEIGHT_RATIO: f32 = 1.45;
const ALERT_MESSAGE_MIN_WIDTH: f32 = 1.0;
const ALERT_MESSAGE_VERTICAL_CENTER_RATIO: f32 = 0.5;
pub(super) const ALERT_MESSAGE_VERTICAL_INSET: f32 = 8.0;

// 几何模块把节点字号投影到 Alert 正文；无效字号退回组件默认值，供文本框与命令共用。
pub(super) fn alert_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        ALERT_FONT_SIZE
    }
}

pub(super) fn alert_message_line_height(font_size: f32) -> f32 {
    font_size * ALERT_MESSAGE_LINE_HEIGHT_RATIO
}

pub(super) fn alert_message_width(available_width: f32) -> f32 {
    available_width.max(ALERT_MESSAGE_MIN_WIDTH)
}

pub(super) fn alert_message_y(rect: &FrameRect, line_height: f32) -> f32 {
    rect.y + (rect.height - line_height).max(0.0) * ALERT_MESSAGE_VERTICAL_CENTER_RATIO
}

pub(super) fn alert_message_content_height(rect: &FrameRect, line_height: f32) -> Option<f32> {
    let content_height = (rect.height - ALERT_MESSAGE_VERTICAL_INSET * 2.0).max(0.0);
    (content_height >= line_height * 2.0).then_some(content_height)
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
