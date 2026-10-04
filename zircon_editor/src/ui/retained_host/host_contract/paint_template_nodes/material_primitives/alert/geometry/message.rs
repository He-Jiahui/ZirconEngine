use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::identity::alert_has_icon;
use super::action::alert_action_width;
use super::metrics::{
    alert_font_size, alert_message_content_height, alert_message_line_height, alert_message_width,
    alert_message_y, ALERT_ICON_EDGE, ALERT_ICON_GAP, ALERT_MESSAGE_VERTICAL_INSET,
    ALERT_PADDING_X,
};

// 正文绘制在图标和操作之间取可用带宽；高 Alert 交由运行时换行，小高度保持单行居中。
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
) -> Option<(FrameRect, f32, f32)> {
    if right <= left {
        return None;
    }
    let font_size = alert_font_size(node);
    let line_height = alert_message_line_height(font_size);
    let content_height = alert_message_content_height(rect, line_height);
    Some((
        FrameRect {
            x: left,
            y: content_height
                .map(|_| rect.y + ALERT_MESSAGE_VERTICAL_INSET)
                .unwrap_or_else(|| alert_message_y(rect, line_height)),
            width: alert_message_width(right - left),
            height: content_height.unwrap_or(line_height),
        },
        font_size,
        line_height,
    ))
}

#[cfg(test)]
#[path = "tests/message.rs"]
mod tests;
