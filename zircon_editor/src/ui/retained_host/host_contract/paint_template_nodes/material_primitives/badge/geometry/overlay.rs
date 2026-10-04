use super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::anchor::badge_anchor_point;
use super::metrics::{
    badge_centered_text_x, badge_centered_text_y, badge_overlay_font_size, badge_overlay_rect,
    badge_overlay_size, badge_overlay_text_line_height, badge_text_width, BADGE_DOT_RADIUS,
    BADGE_STANDARD_RADIUS,
};
use super::model::BadgeTextFrame;

// 覆盖层先按实测计数字宽定尺寸，再围绕 variant 锚点布局；圆点跳过文字测量。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    display: &str,
    dot: bool,
) -> FrameRect {
    let measured_text_width = if dot {
        0.0
    } else {
        measure_runtime_text_width(display, badge_overlay_font_size())
    };
    let (width, height) = badge_overlay_size(measured_text_width, dot);
    let (anchor_x, anchor_y) = badge_anchor_point(node, rect);
    badge_overlay_rect(anchor_x, anchor_y, width, height)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_radius(
    dot: bool,
) -> f32 {
    if dot {
        BADGE_DOT_RADIUS
    } else {
        BADGE_STANDARD_RADIUS
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_text_frame(
    display: &str,
    rect: &FrameRect,
) -> BadgeTextFrame {
    let font_size = badge_overlay_font_size();
    let line_height = badge_overlay_text_line_height(font_size);
    let text_width = badge_text_width(measure_runtime_text_width(display, font_size), rect.width);
    BadgeTextFrame {
        rect: FrameRect {
            x: badge_centered_text_x(rect, text_width),
            y: badge_centered_text_y(rect, line_height),
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    }
}

#[cfg(test)]
#[path = "tests/overlay.rs"]
mod tests;
