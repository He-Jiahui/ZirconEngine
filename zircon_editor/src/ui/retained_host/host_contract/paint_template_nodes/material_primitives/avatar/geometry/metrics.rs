pub(super) use super::super::super::bounded_extent as avatar_bounded_extent;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(super) const AVATAR_DEFAULT_EDGE: f32 = 40.0;
pub(super) const AVATAR_TEXT_FONT_RATIO: f32 = 0.5;
pub(super) const AVATAR_MIN_FONT_SIZE: f32 = 8.0;
pub(super) const AVATAR_FALLBACK_SCALE: f32 = 0.75;

const AVATAR_TEXT_CENTER_RATIO: f32 = 0.5;

pub(super) fn avatar_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let available_edge = avatar_bounded_extent(rect.width).min(avatar_bounded_extent(rect.height));
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        available_edge * AVATAR_TEXT_FONT_RATIO
    };
    requested.max(AVATAR_MIN_FONT_SIZE).min(available_edge)
}

pub(super) fn avatar_text_line_height(font_size: f32) -> f32 {
    font_size
}

pub(super) fn avatar_text_width(measured_width: f32, available_width: f32) -> f32 {
    measured_width
        .is_finite()
        .then_some(measured_width.max(0.0))
        .unwrap_or(0.0)
        .min(avatar_bounded_extent(available_width))
}

pub(super) fn avatar_centered_text_x(rect: &FrameRect, text_width: f32) -> f32 {
    rect.x + (avatar_bounded_extent(rect.width) - text_width).max(0.0) * AVATAR_TEXT_CENTER_RATIO
}

pub(super) fn avatar_centered_text_y(rect: &FrameRect, line_height: f32) -> f32 {
    rect.y + (avatar_bounded_extent(rect.height) - line_height).max(0.0) * AVATAR_TEXT_CENTER_RATIO
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
    fn avatar_text_width_clamps_to_avatar_bounds() {
        assert!((avatar_text_width(80.0, 32.0) - 32.0).abs() <= 0.01);
        assert!((avatar_text_width(0.0, 32.0) - 0.0).abs() <= 0.01);
        assert!((avatar_text_width(12.0, 0.4) - 0.4).abs() <= 0.01);
    }
}
