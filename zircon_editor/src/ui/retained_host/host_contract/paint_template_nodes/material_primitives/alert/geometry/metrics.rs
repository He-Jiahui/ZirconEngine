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
mod tests {
    use super::*;

    fn node(font_size: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            font_size,
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn alert_message_metrics_project_font_line_height_and_y() {
        let rect = FrameRect {
            x: 0.0,
            y: 4.0,
            width: 200.0,
            height: 48.0,
        };
        let line_height = alert_message_line_height(alert_font_size(&node(13.0)));

        assert!((line_height - 18.85).abs() <= 0.01);
        assert!((alert_message_y(&rect, line_height) - 18.575).abs() <= 0.01);
    }

    #[test]
    fn alert_message_width_uses_available_space_with_a_minimum() {
        assert!((alert_message_width(44.0) - 44.0).abs() <= 0.01);
        assert!((alert_message_width(0.0) - 1.0).abs() <= 0.01);
    }

    #[test]
    fn alert_message_content_height_requires_room_for_two_lines() {
        let line_height = 18.0;
        let compact = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 48.0,
        };
        let tall = FrameRect {
            height: 64.0,
            ..compact.clone()
        };

        assert_eq!(alert_message_content_height(&compact, line_height), None);
        assert_eq!(alert_message_content_height(&tall, line_height), Some(48.0));
    }
}
