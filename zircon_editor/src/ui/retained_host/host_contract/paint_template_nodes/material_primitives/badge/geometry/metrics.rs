use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
pub(super) use super::super::super::bounded_extent as badge_bounded_extent;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

pub(super) const BADGE_STANDARD_HEIGHT: f32 = 20.0;
pub(super) const BADGE_STANDARD_MIN_WIDTH: f32 = 20.0;
pub(super) const BADGE_STANDARD_PADDING_X: f32 = 6.0;
pub(super) const BADGE_DOT_EDGE: f32 = 8.0;
pub(super) const BADGE_STANDARD_RADIUS: f32 = 10.0;
pub(super) const BADGE_DOT_RADIUS: f32 = 4.0;
pub(super) const BADGE_FONT_SIZE: f32 = 12.0;
pub(super) const BADGE_ROOT_TEXT_INSET_X: f32 = 8.0;
pub(super) const BADGE_CIRCULAR_OFFSET_RATIO: f32 = 0.14;

const BADGE_MIN_TEXT_EXTENT: f32 = 1.0;
const BADGE_CENTER_RATIO: f32 = 0.5;

pub(super) fn badge_root_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    badge_root_font_size_from_host(node, rect, current_host_metrics())
}

fn badge_root_font_size_from_host(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: HostControlMetrics,
) -> f32 {
    let available_edge = badge_bounded_extent(rect.width).min(badge_bounded_extent(rect.height));
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size.min(available_edge)
    } else {
        badge_bounded_extent(metrics.font_body).min(available_edge)
    }
}

pub(super) fn badge_text_line_height(font_size: f32, rect: &FrameRect) -> f32 {
    badge_bounded_extent(current_host_metrics().line_height(font_size))
        .min(badge_bounded_extent(rect.height))
}

pub(super) fn badge_root_available_text_width(rect: &FrameRect) -> f32 {
    let width = badge_bounded_extent(rect.width);
    (width - badge_root_text_inset(rect) * 2.0).max(0.0)
}

pub(super) fn badge_text_width(measured_width: f32, available_width: f32) -> f32 {
    measured_width
        .is_finite()
        .then_some(measured_width.max(0.0))
        .unwrap_or(0.0)
        .min(badge_bounded_extent(available_width))
}

pub(super) fn badge_root_text_x(rect: &FrameRect) -> f32 {
    rect.x + badge_root_text_inset(rect)
}

pub(super) fn badge_centered_text_x(rect: &FrameRect, text_width: f32) -> f32 {
    rect.x + (badge_bounded_extent(rect.width) - text_width).max(0.0) * BADGE_CENTER_RATIO
}

pub(super) fn badge_centered_text_y(rect: &FrameRect, line_height: f32) -> f32 {
    rect.y + (badge_bounded_extent(rect.height) - line_height).max(0.0) * BADGE_CENTER_RATIO
}

fn badge_root_text_inset(rect: &FrameRect) -> f32 {
    BADGE_ROOT_TEXT_INSET_X.min(badge_bounded_extent(rect.width) * 0.5)
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
    let width = badge_bounded_extent(width).max(BADGE_MIN_TEXT_EXTENT);
    let height = badge_bounded_extent(height).max(BADGE_MIN_TEXT_EXTENT);
    FrameRect {
        x: anchor_x - width * BADGE_CENTER_RATIO,
        y: anchor_y - height * BADGE_CENTER_RATIO,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

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
        let font_size = badge_root_font_size(&node(12.0), &rect);

        assert!((font_size - 12.0).abs() <= 0.01);
        assert!((badge_text_line_height(font_size, &rect) - 14.4).abs() <= 0.01);
        assert!((badge_root_available_text_width(&rect) - 144.0).abs() <= 0.01);
        assert!((badge_root_text_x(&rect) - 10.0).abs() <= 0.01);
    }

    #[test]
    fn badge_root_font_size_tracks_host_typography_and_tight_bounds() {
        let mut host = METRICS;
        host.font_body = 11.0;
        let wide = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 24.0,
        };
        let tight = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 6.0,
        };
        let node = node(0.0);

        assert_eq!(badge_root_font_size_from_host(&node, &wide, host), 11.0);
        assert_eq!(badge_root_font_size_from_host(&node, &tight, host), 4.0);
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
    fn badge_overlay_preserves_fractional_post_dpi_geometry() {
        let rect = badge_overlay_rect(40.25, 30.75, 36.5, 20.5);

        assert_eq!(rect.x, 22.0);
        assert_eq!(rect.y, 20.5);
        assert_eq!(rect.width, 36.5);
        assert_eq!(rect.height, 20.5);
    }

    #[test]
    fn badge_text_width_clamps_to_available_bounds() {
        assert!((badge_text_width(80.0, 20.0) - 20.0).abs() <= 0.01);
        assert!((badge_text_width(0.0, 20.0) - 0.0).abs() <= 0.01);
    }
}
