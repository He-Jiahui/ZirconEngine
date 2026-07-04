use super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::anchor::badge_anchor_point;
use super::metrics::{
    BADGE_DOT_EDGE, BADGE_DOT_RADIUS, BADGE_FONT_SIZE, BADGE_STANDARD_HEIGHT,
    BADGE_STANDARD_MIN_WIDTH, BADGE_STANDARD_PADDING_X, BADGE_STANDARD_RADIUS,
};
use super::model::BadgeTextFrame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    display: &str,
    dot: bool,
) -> FrameRect {
    let (width, height) = if dot {
        (BADGE_DOT_EDGE, BADGE_DOT_EDGE)
    } else {
        (
            (measure_runtime_text_width(display, BADGE_FONT_SIZE) + BADGE_STANDARD_PADDING_X * 2.0)
                .max(BADGE_STANDARD_MIN_WIDTH),
            BADGE_STANDARD_HEIGHT,
        )
    };
    let (anchor_x, anchor_y) = badge_anchor_point(node, rect);
    FrameRect {
        x: (anchor_x - width * 0.5).round(),
        y: (anchor_y - height * 0.5).round(),
        width: width.round().max(1.0),
        height: height.round().max(1.0),
    }
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
    let line_height = BADGE_FONT_SIZE;
    let text_width = measure_runtime_text_width(display, BADGE_FONT_SIZE)
        .min(rect.width)
        .max(1.0);
    BadgeTextFrame {
        rect: FrameRect {
            x: rect.x + (rect.width - text_width).max(0.0) * 0.5,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        font_size: BADGE_FONT_SIZE,
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
        let expected_width = (measure_runtime_text_width(display, BADGE_FONT_SIZE)
            + BADGE_STANDARD_PADDING_X * 2.0)
            .max(BADGE_STANDARD_MIN_WIDTH)
            .round()
            .max(1.0);

        assert!((frame.width - expected_width).abs() <= 0.01);
        let old_heuristic_width = (display.chars().count() as f32 * BADGE_FONT_SIZE * 0.56
            + BADGE_STANDARD_PADDING_X * 2.0)
            .max(BADGE_STANDARD_MIN_WIDTH)
            .round()
            .max(1.0);
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
        let expected_width = measure_runtime_text_width(display, BADGE_FONT_SIZE)
            .min(overlay.width)
            .max(1.0);

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
