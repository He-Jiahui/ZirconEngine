use crate::ui::retained_host::host_contract::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};

use super::metrics::{AVATAR_MIN_FONT_SIZE, AVATAR_TEXT_FONT_RATIO};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> (FrameRect, f32, f32) {
    let font_size = avatar_font_size(node, rect);
    let line_height = font_size;
    let text_width = measure_runtime_text_width(label, font_size)
        .min(rect.width)
        .max(1.0);
    (
        FrameRect {
            x: rect.x + (rect.width - text_width).max(0.0) * 0.5,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    )
}

fn avatar_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        rect.width.min(rect.height) * AVATAR_TEXT_FONT_RATIO
    };
    requested.max(AVATAR_MIN_FONT_SIZE).min(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(width: f32) -> FrameRect {
        FrameRect {
            x: 4.0,
            y: 6.0,
            width,
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
    fn avatar_text_frame_uses_runtime_text_width() {
        let node = node(18.0);
        let rect = rect(180.0);
        let label = "WWW iii";
        let (frame, font_size, _) = avatar_text_frame(&node, &rect, label);
        let expected_width = measure_runtime_text_width(label, font_size)
            .min(rect.width)
            .max(1.0);

        assert!((frame.width - expected_width).abs() <= 0.01);
        assert!((frame.x - (rect.x + (rect.width - expected_width) * 0.5)).abs() <= 0.01);
        let old_heuristic_width = label.chars().count() as f32 * font_size * 0.58;
        assert!((expected_width - old_heuristic_width).abs() > 0.25);
    }

    #[test]
    fn avatar_text_frame_clamps_measured_width_to_avatar_width() {
        let node = node(18.0);
        let rect = rect(32.0);
        let (frame, _, _) = avatar_text_frame(&node, &rect, "Long avatar label");

        assert!((frame.width - rect.width).abs() <= 0.01);
        assert!((frame.x - rect.x).abs() <= 0.01);
    }
}
