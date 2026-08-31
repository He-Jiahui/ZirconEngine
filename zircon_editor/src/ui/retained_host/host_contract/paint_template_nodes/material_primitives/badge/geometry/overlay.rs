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
mod tests {
    use super::*;

    fn rect() -> FrameRect {
        FrameRect {
            x: 20.0,
            y: 30.0,
            width: 80.0,
            height: 40.0,
        }
    }

    #[test]
    fn badge_overlay_frame_uses_runtime_text_width_with_padding() {
        let node = TemplatePaneNodeData::default();
        let display = "WWW iii";
        let frame = badge_overlay_frame(&node, &rect(), display, false);
        let (expected_width, _) = badge_overlay_size(
            measure_runtime_text_width(display, badge_overlay_font_size()),
            false,
        );

        assert!((frame.width - expected_width).abs() <= 0.01);
        let (old_heuristic_width, _) = badge_overlay_size(
            display.chars().count() as f32 * badge_overlay_font_size() * 0.56,
            false,
        );
        assert!((expected_width - old_heuristic_width).abs() > 0.25);
    }

    #[test]
    fn badge_overlay_text_frame_uses_runtime_text_width() {
        let display = "WWW iii";
        let overlay = FrameRect {
            width: 160.0,
            ..rect()
        };
        let text_frame = badge_overlay_text_frame(display, &overlay);
        let expected_width = badge_text_width(
            measure_runtime_text_width(display, badge_overlay_font_size()),
            overlay.width,
        );

        assert!((text_frame.rect.width - expected_width).abs() <= 0.01);
    }

    #[test]
    fn badge_overlay_text_frame_clamps_measured_width_to_overlay_width() {
        let overlay = FrameRect {
            width: 20.0,
            ..rect()
        };
        let text_frame = badge_overlay_text_frame("Long badge overlay", &overlay);

        assert!((text_frame.rect.width - overlay.width).abs() <= 0.01);
    }
}
