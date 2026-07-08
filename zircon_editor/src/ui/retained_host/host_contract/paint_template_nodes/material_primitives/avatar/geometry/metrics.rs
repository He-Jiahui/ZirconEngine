use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(super) const AVATAR_DEFAULT_EDGE: f32 = 40.0;
pub(super) const AVATAR_ROUNDED_RADIUS: f32 = 4.0;
pub(super) const AVATAR_TEXT_FONT_RATIO: f32 = 0.5;
pub(super) const AVATAR_MIN_FONT_SIZE: f32 = 8.0;
pub(super) const AVATAR_FALLBACK_SCALE: f32 = 0.75;

const AVATAR_MIN_TEXT_WIDTH: f32 = 1.0;
const AVATAR_TEXT_CENTER_RATIO: f32 = 0.5;

pub(super) fn avatar_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        rect.width.min(rect.height) * AVATAR_TEXT_FONT_RATIO
    };
    requested.max(AVATAR_MIN_FONT_SIZE).min(rect.height)
}

pub(super) fn avatar_text_line_height(font_size: f32) -> f32 {
    font_size
}

pub(super) fn avatar_text_width(measured_width: f32, available_width: f32) -> f32 {
    measured_width
        .min(available_width)
        .max(AVATAR_MIN_TEXT_WIDTH)
}

pub(super) fn avatar_centered_text_x(rect: &FrameRect, text_width: f32) -> f32 {
    rect.x + (rect.width - text_width).max(0.0) * AVATAR_TEXT_CENTER_RATIO
}

pub(super) fn avatar_centered_text_y(rect: &FrameRect, line_height: f32) -> f32 {
    rect.y + (rect.height - line_height).max(0.0) * AVATAR_TEXT_CENTER_RATIO
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
    fn avatar_text_metrics_project_font_width_and_centering() {
        let rect = FrameRect {
            x: 4.0,
            y: 6.0,
            width: 48.0,
            height: 48.0,
        };
        let font_size = avatar_font_size(&node(0.0), &rect);
        let text_width = avatar_text_width(18.0, rect.width);

        assert!((font_size - 24.0).abs() <= 0.01);
        assert!((avatar_text_line_height(font_size) - 24.0).abs() <= 0.01);
        assert!((avatar_centered_text_x(&rect, text_width) - 19.0).abs() <= 0.01);
        assert!((avatar_centered_text_y(&rect, font_size) - 18.0).abs() <= 0.01);
    }

    #[test]
    fn avatar_text_width_clamps_to_avatar_and_minimum() {
        assert!((avatar_text_width(80.0, 32.0) - 32.0).abs() <= 0.01);
        assert!((avatar_text_width(0.0, 32.0) - 1.0).abs() <= 0.01);
    }
}
