pub(super) use super::super::super::bounded_extent as avatar_bounded_extent;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(super) const AVATAR_DEFAULT_EDGE: f32 = 40.0;
pub(super) const AVATAR_TEXT_FONT_RATIO: f32 = 0.5;
pub(super) const AVATAR_MIN_FONT_SIZE: f32 = 8.0;
pub(super) const AVATAR_FALLBACK_SCALE: f32 = 0.75;

const AVATAR_TEXT_CENTER_RATIO: f32 = 0.5;

// 文本框消费宿主字号或按头像尺寸推导字号，最终受可用边长限制。
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
#[path = "tests/metrics.rs"]
mod tests;
