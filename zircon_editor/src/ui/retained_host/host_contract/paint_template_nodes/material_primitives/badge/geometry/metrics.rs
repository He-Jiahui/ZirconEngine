use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
pub(super) use super::super::super::bounded_extent as badge_bounded_extent;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

// 根文字跟随宿主字体且受根框约束，计数层使用独立的固定组件度量。
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
#[path = "tests/metrics.rs"]
mod tests;
