use super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::metrics::{BADGE_ROOT_FONT_SIZE, BADGE_ROOT_TEXT_INSET_X};
use super::model::BadgeTextFrame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> BadgeTextFrame {
    let font_size = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        BADGE_ROOT_FONT_SIZE
    };
    let line_height = font_size * 1.2;
    let available_width = (rect.width - BADGE_ROOT_TEXT_INSET_X * 2.0).max(1.0);
    let text_width = measure_runtime_text_width(label, font_size)
        .min(available_width)
        .max(1.0);
    BadgeTextFrame {
        rect: FrameRect {
            x: rect.x + BADGE_ROOT_TEXT_INSET_X,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(width: f32) -> FrameRect {
        FrameRect {
            x: 2.0,
            y: 4.0,
            width,
            height: 28.0,
        }
    }

    fn node(font_size: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            font_size,
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn badge_root_text_frame_uses_runtime_text_width() {
        let node = node(12.0);
        let rect = rect(160.0);
        let label = "WWW iii";
        let text_frame = badge_root_text_frame(&node, &rect, label);
        let available_width = rect.width - BADGE_ROOT_TEXT_INSET_X * 2.0;
        let expected_width = measure_runtime_text_width(label, text_frame.font_size)
            .min(available_width)
            .max(1.0);

        assert!((text_frame.rect.width - expected_width).abs() <= 0.01);
        let old_heuristic_width = label.chars().count() as f32 * text_frame.font_size * 0.56;
        assert!((expected_width - old_heuristic_width).abs() > 0.25);
    }

    #[test]
    fn badge_root_text_frame_clamps_measured_width_to_inset_width() {
        let node = node(12.0);
        let rect = rect(36.0);
        let text_frame = badge_root_text_frame(&node, &rect, "Long badge label");
        let available_width = rect.width - BADGE_ROOT_TEXT_INSET_X * 2.0;

        assert!((text_frame.rect.width - available_width).abs() <= 0.01);
    }
}
