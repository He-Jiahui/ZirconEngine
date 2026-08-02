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

pub(super) fn alert_message_width(measured_width: f32, available_width: f32) -> f32 {
    measured_width
        .min(available_width)
        .max(ALERT_MESSAGE_MIN_WIDTH)
}

pub(super) fn alert_message_y(rect: &FrameRect, line_height: f32) -> f32 {
    rect.y + (rect.height - line_height).max(0.0) * ALERT_MESSAGE_VERTICAL_CENTER_RATIO
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
    fn alert_message_width_clamps_to_available_and_minimum() {
        assert!((alert_message_width(80.0, 44.0) - 44.0).abs() <= 0.01);
        assert!((alert_message_width(0.0, 44.0) - 1.0).abs() <= 0.01);
    }
}
