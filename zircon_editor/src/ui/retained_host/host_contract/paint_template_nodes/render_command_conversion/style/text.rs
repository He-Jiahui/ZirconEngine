use zircon_runtime::ui::surface::measure_text_size;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRunPaintStyle,
};

use super::super::super::super::data::FrameRect;

const STRONG_TEXT_FONT_WEIGHT_THRESHOLD: u16 = 600;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn aligned_text_x(
    frame: &FrameRect,
    text: &str,
    style: &UiResolvedStyle,
) -> f32 {
    let measured_width = measure_text_size(text, style).width.max(0.0);
    match resolved_text_align(style.text_align, style.text_direction) {
        UiTextAlign::Left | UiTextAlign::Justify => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - measured_width).max(0.0) * 0.5,
        UiTextAlign::Right => frame.x + (frame.width - measured_width).max(0.0),
        UiTextAlign::Start | UiTextAlign::End => unreachable!("resolved align is physical"),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_paint_style_from_resolved_style(
    style: &UiResolvedStyle,
) -> UiTextRunPaintStyle {
    text_paint_style_from_font_weight(style.font_weight)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_paint_style_from_font_weight(
    font_weight: u16,
) -> UiTextRunPaintStyle {
    UiTextRunPaintStyle {
        strong: UiResolvedStyle::normalized_font_weight(font_weight)
            >= STRONG_TEXT_FONT_WEIGHT_THRESHOLD,
        ..UiTextRunPaintStyle::default()
    }
}

fn resolved_text_align(align: UiTextAlign, direction: UiTextDirection) -> UiTextAlign {
    match align {
        UiTextAlign::Start => match direction {
            UiTextDirection::RightToLeft => UiTextAlign::Right,
            UiTextDirection::Auto | UiTextDirection::LeftToRight | UiTextDirection::Mixed => {
                UiTextAlign::Left
            }
        },
        UiTextAlign::End => match direction {
            UiTextDirection::RightToLeft => UiTextAlign::Left,
            UiTextDirection::Auto | UiTextDirection::LeftToRight | UiTextDirection::Mixed => {
                UiTextAlign::Right
            }
        },
        UiTextAlign::Left | UiTextAlign::Center | UiTextAlign::Right | UiTextAlign::Justify => {
            align
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::ui::surface::measure_text_size;

    fn frame() -> FrameRect {
        FrameRect {
            x: 10.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        }
    }

    fn style(text_align: UiTextAlign, text_direction: UiTextDirection) -> UiResolvedStyle {
        UiResolvedStyle {
            text_align,
            text_direction,
            font_size: 10.0,
            ..UiResolvedStyle::default()
        }
    }

    #[test]
    fn aligned_text_x_resolves_logical_start_end_against_text_direction() {
        let frame = frame();
        let left = frame.x;
        let right = frame.x + frame.width
            - measure_text_size(
                "abc",
                &style(UiTextAlign::Right, UiTextDirection::LeftToRight),
            )
            .width
            .max(0.0);

        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::Start, UiTextDirection::LeftToRight)
            ),
            left
        );
        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::End, UiTextDirection::LeftToRight)
            ),
            right
        );
        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::Start, UiTextDirection::RightToLeft)
            ),
            right
        );
        assert_eq!(
            aligned_text_x(
                &frame,
                "abc",
                &style(UiTextAlign::End, UiTextDirection::RightToLeft)
            ),
            left
        );
    }

    #[test]
    fn aligned_text_x_keeps_justify_at_line_start_for_runtime_spacing() {
        let frame = frame();

        assert_eq!(
            aligned_text_x(
                &frame,
                "a b",
                &style(UiTextAlign::Justify, UiTextDirection::LeftToRight)
            ),
            frame.x
        );
    }

    #[test]
    fn aligned_text_x_uses_runtime_surface_measurement() {
        let frame = frame();
        let style = style(UiTextAlign::Center, UiTextDirection::LeftToRight);
        let text = "a\u{0301}b";
        let runtime_width = measure_text_size(text, &style).width;
        let legacy_width = text.chars().count() as f32 * (style.font_size * 0.5);

        assert_ne!(runtime_width.round(), legacy_width.round());
        assert_eq!(
            aligned_text_x(&frame, text, &style),
            frame.x + (frame.width - runtime_width).max(0.0) * 0.5
        );
    }
}
