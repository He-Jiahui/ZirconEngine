use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) const BADGE_STANDARD_HEIGHT: f32 = 20.0;
pub(super) const BADGE_STANDARD_MIN_WIDTH: f32 = 20.0;
pub(super) const BADGE_STANDARD_PADDING_X: f32 = 6.0;
pub(super) const BADGE_DOT_EDGE: f32 = 8.0;
pub(super) const BADGE_STANDARD_RADIUS: f32 = 10.0;
pub(super) const BADGE_DOT_RADIUS: f32 = 4.0;
pub(super) const BADGE_FONT_SIZE: f32 = 12.0;
pub(super) const BADGE_ROOT_FONT_SIZE: f32 = 12.0;
pub(super) const BADGE_ROOT_TEXT_INSET_X: f32 = 8.0;
pub(super) const BADGE_CIRCULAR_OFFSET_RATIO: f32 = 0.14;

const BADGE_TEXT_LINE_HEIGHT_RATIO: f32 = 1.2;
const BADGE_MIN_TEXT_EXTENT: f32 = 1.0;
const BADGE_CENTER_RATIO: f32 = 0.5;

pub(super) fn badge_root_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        BADGE_ROOT_FONT_SIZE
    }
}

pub(super) fn badge_text_line_height(font_size: f32) -> f32 {
    font_size * BADGE_TEXT_LINE_HEIGHT_RATIO
}

pub(super) fn badge_root_available_text_width(rect: &FrameRect) -> f32 {
    (rect.width - BADGE_ROOT_TEXT_INSET_X * 2.0).max(BADGE_MIN_TEXT_EXTENT)
}

pub(super) fn badge_text_width(measured_width: f32, available_width: f32) -> f32 {
    measured_width
        .min(available_width)
        .max(BADGE_MIN_TEXT_EXTENT)
}

pub(super) fn badge_root_text_x(rect: &FrameRect) -> f32 {
    rect.x + BADGE_ROOT_TEXT_INSET_X
}

pub(super) fn badge_centered_text_x(rect: &FrameRect, text_width: f32) -> f32 {
    rect.x + (rect.width - text_width).max(0.0) * BADGE_CENTER_RATIO
}

pub(super) fn badge_centered_text_y(rect: &FrameRect, line_height: f32) -> f32 {
    rect.y + (rect.height - line_height).max(0.0) * BADGE_CENTER_RATIO
}

pub(super) fn badge_overlay_font_size() -> f32 {
    BADGE_FONT_SIZE
}

pub(super) fn badge_overlay_text_line_height(font_size: f32) -> f32 {
    font_size
}

pub(super) fn badge_overlay_size(measured_text_width: f32, dot: bool) -> (f32, f32) {
    if dot {
        (BADGE_DOT_EDGE, BADGE_DOT_EDGE)
    } else {
        (
            (measured_text_width + BADGE_STANDARD_PADDING_X * 2.0).max(BADGE_STANDARD_MIN_WIDTH),
            BADGE_STANDARD_HEIGHT,
        )
    }
}

pub(super) fn badge_overlay_rect(
    anchor_x: f32,
    anchor_y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    FrameRect {
        x: (anchor_x - width * BADGE_CENTER_RATIO).round(),
        y: (anchor_y - height * BADGE_CENTER_RATIO).round(),
        width: width.round().max(BADGE_MIN_TEXT_EXTENT),
        height: height.round().max(BADGE_MIN_TEXT_EXTENT),
    }
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
    fn badge_root_text_metrics_project_font_line_height_and_width() {
        let rect = FrameRect {
            x: 2.0,
            y: 4.0,
            width: 160.0,
            height: 28.0,
        };
        let font_size = badge_root_font_size(&node(12.0));

        assert!((font_size - 12.0).abs() <= 0.01);
        assert!((badge_text_line_height(font_size) - 14.4).abs() <= 0.01);
        assert!((badge_root_available_text_width(&rect) - 144.0).abs() <= 0.01);
        assert!((badge_root_text_x(&rect) - 10.0).abs() <= 0.01);
    }

    #[test]
    fn badge_overlay_metrics_project_text_size_and_rect() {
        let (width, height) = badge_overlay_size(24.0, false);
        let rect = badge_overlay_rect(40.0, 30.0, width, height);

        assert!((width - 36.0).abs() <= 0.01);
        assert!((height - 20.0).abs() <= 0.01);
        assert!((rect.x - 22.0).abs() <= 0.01);
        assert!((rect.y - 20.0).abs() <= 0.01);
    }

    #[test]
    fn badge_text_width_clamps_to_available_and_minimum() {
        assert!((badge_text_width(80.0, 20.0) - 20.0).abs() <= 0.01);
        assert!((badge_text_width(0.0, 20.0) - 1.0).abs() <= 0.01);
    }
}
