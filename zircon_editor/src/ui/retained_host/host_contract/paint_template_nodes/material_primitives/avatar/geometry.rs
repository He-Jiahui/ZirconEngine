use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::component_variant_contains;

const AVATAR_DEFAULT_EDGE: f32 = 40.0;
const AVATAR_ROUNDED_RADIUS: f32 = 4.0;
const AVATAR_TEXT_FONT_RATIO: f32 = 0.5;
const AVATAR_MIN_FONT_SIZE: f32 = 8.0;
const AVATAR_TEXT_WIDTH_RATIO: f32 = 0.58;

pub(super) fn avatar_frame(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).min(AVATAR_DEFAULT_EDGE).round();
    let size = size.max(1.0);
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - size).max(0.0) * 0.5).round(),
        width: size,
        height: size,
    }
}

pub(super) fn avatar_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    if component_variant_contains(node, "rounded") {
        let configured = node
            .corner_radius
            .max(node.button_style.element.corner_radius)
            .max(0.0);
        return if configured > 0.0 {
            configured
        } else {
            AVATAR_ROUNDED_RADIUS
        };
    }
    rect.width.min(rect.height) * 0.5
}

pub(super) fn avatar_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> (FrameRect, f32, f32) {
    let font_size = avatar_font_size(node, rect);
    let line_height = font_size;
    let text_width = (label.chars().count() as f32 * font_size * AVATAR_TEXT_WIDTH_RATIO)
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

pub(super) fn centered_child_rect(rect: &FrameRect, scale: f32) -> FrameRect {
    let size = (rect.width.min(rect.height) * scale.clamp(0.0, 1.0)).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}

fn avatar_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        rect.width.min(rect.height) * AVATAR_TEXT_FONT_RATIO
    };
    requested.max(AVATAR_MIN_FONT_SIZE).min(rect.height)
}
