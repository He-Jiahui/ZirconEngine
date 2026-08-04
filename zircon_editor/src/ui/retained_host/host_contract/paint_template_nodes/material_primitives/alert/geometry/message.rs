use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::identity::alert_has_icon;
use super::action::alert_action_width;
use super::metrics::{
    alert_font_size, alert_message_content_height, alert_message_line_height, alert_message_width,
    alert_message_y, ALERT_ICON_EDGE, ALERT_ICON_GAP, ALERT_MESSAGE_VERTICAL_INSET,
    ALERT_PADDING_X,
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
mod tests {
    use super::*;

    fn rect() -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 260.0,
            height: 48.0,
        }
    }

    fn node(font_size: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            font_size,
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn alert_message_frame_uses_the_available_message_band() {
        let node = node(13.0);
        let frame = alert_message_frame(&node, &rect(), 0.0, 260.0)
            .expect("alert message has enough horizontal space")
            .0;

        assert!((frame.width - 260.0).abs() <= 0.01);
    }

    #[test]
    fn alert_message_frame_clamps_to_available_message_space() {
        let node = node(13.0);
        let frame = alert_message_frame(&node, &rect(), 0.0, 44.0)
            .expect("alert message has enough horizontal space")
            .0;

        assert!((frame.width - 44.0).abs() <= 0.01);
    }

    #[test]
    fn tall_alert_message_frame_reserves_a_multiline_content_band() {
        let node = node(13.0);
        let alert = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 260.0,
            height: 88.0,
        };
        let frame = alert_message_frame(&node, &alert, 16.0, 244.0)
            .expect("tall alert has message space")
            .0;
        let line_height = alert_message_line_height(alert_font_size(&node));

        assert!(frame.height >= line_height * 2.0);
        assert!(frame.y >= alert.y);
        assert!(frame.bottom() <= alert.bottom());
    }
}
