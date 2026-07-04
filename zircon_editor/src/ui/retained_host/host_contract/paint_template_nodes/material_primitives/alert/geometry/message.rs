use crate::ui::retained_host::host_contract::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};

use super::super::identity::alert_has_icon;
use super::action::alert_action_width;
use super::metrics::{ALERT_FONT_SIZE, ALERT_ICON_EDGE, ALERT_ICON_GAP, ALERT_PADDING_X};

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
    let width = measure_runtime_text_width(message, font_size)
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
    fn alert_message_frame_uses_runtime_text_width() {
        let node = node(13.0);
        let message = "WWW iii";
        let frame = alert_message_frame(&node, &rect(), 0.0, 260.0, message)
            .expect("alert message has enough horizontal space")
            .0;
        let expected_width = measure_runtime_text_width(message, 13.0)
            .min(260.0)
            .max(1.0);

        assert!((frame.width - expected_width).abs() <= 0.01);
        let old_heuristic_width = message.chars().count() as f32 * 13.0 * 0.56;
        assert!((expected_width - old_heuristic_width).abs() > 0.25);
    }

    #[test]
    fn alert_message_frame_clamps_measured_width_to_available_space() {
        let node = node(13.0);
        let frame = alert_message_frame(&node, &rect(), 0.0, 44.0, "Long alert message")
            .expect("alert message has enough horizontal space")
            .0;

        assert!((frame.width - 44.0).abs() <= 0.01);
    }
}
