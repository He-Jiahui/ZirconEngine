use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{BADGE_ROOT_FONT_SIZE, BADGE_ROOT_TEXT_INSET_X, BADGE_ROOT_TEXT_WIDTH_RATIO};
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
    let text_width = (label.chars().count() as f32 * font_size * BADGE_ROOT_TEXT_WIDTH_RATIO)
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
