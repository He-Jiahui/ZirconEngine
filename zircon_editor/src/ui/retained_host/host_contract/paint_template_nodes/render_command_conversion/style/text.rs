use zircon_runtime::ui::surface::measure_text_size;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRunPaintStyle,
};

use super::super::super::super::{
    data::FrameRect,
    paint_text::{font_face_for_paint_style, runtime_text_style_for_face},
};

mod metrics;

use self::metrics::{center_aligned_text_x, measured_text_width, right_aligned_text_x};

const STRONG_TEXT_FONT_WEIGHT_THRESHOLD: u16 = 600;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn aligned_text_x(
    frame: &FrameRect,
    text: &str,
    style: &UiResolvedStyle,
) -> f32 {
    let measure_style = retained_runtime_measure_style(style);
    let measured_width = measured_text_width(measure_text_size(text, &measure_style).width);
    match resolved_text_align(style.text_align, style.text_direction) {
        UiTextAlign::Left | UiTextAlign::Justify => frame.x,
        UiTextAlign::Center => center_aligned_text_x(frame.x, frame.width, measured_width),
        UiTextAlign::Right => right_aligned_text_x(frame.x, frame.width, measured_width),
        UiTextAlign::Start | UiTextAlign::End => frame.x,
    }
}

fn retained_runtime_measure_style(style: &UiResolvedStyle) -> UiResolvedStyle {
    let paint_style = text_paint_style_from_resolved_style(style);
    let mut measure_style = runtime_text_style_for_face(
        font_face_for_paint_style(paint_style),
        style.font_size,
        style.line_height,
        style.wrap,
        style.text_overflow,
    );
    measure_style.text_align = style.text_align;
    measure_style.text_direction = style.text_direction;
    measure_style.text_writing_mode = style.text_writing_mode;
    measure_style.text_render_mode = style.text_render_mode;
    measure_style
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
    use super::super::super::super::super::paint_text::{
        HostTextFontFace, runtime_font_family_for_face,
    };
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
        let right_style = style(UiTextAlign::Right, UiTextDirection::LeftToRight);
        let left = frame.x;
        let right = frame.x + frame.width
            - measured_text_width(
                measure_text_size("abc", &retained_runtime_measure_style(&right_style)).width,
            );

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
        let measure_style = retained_runtime_measure_style(&style);
        let runtime_width = measure_text_size(text, &measure_style).width;
        let legacy_width = text.chars().count() as f32 * (style.font_size * 0.5);

        assert_ne!(runtime_width.round(), legacy_width.round());
        assert_eq!(
            aligned_text_x(&frame, text, &style),
            center_aligned_text_x(frame.x, frame.width, runtime_width)
        );
    }

    #[test]
    fn aligned_text_x_uses_retained_resolved_font_family_for_measurement() {
        let frame = frame();
        let mut style = style(UiTextAlign::Right, UiTextDirection::LeftToRight);
        style.font_family = Some("serif".to_string());
        let text = "editor base.zui";
        let measure_style = retained_runtime_measure_style(&style);
        let runtime_width = measured_text_width(measure_text_size(text, &measure_style).width);

        assert_eq!(
            measure_style.font_family.as_deref(),
            Some(runtime_font_family_for_face(HostTextFontFace::Ui))
        );
        assert_ne!(measure_style.font_family, style.font_family);
        assert_eq!(
            aligned_text_x(&frame, text, &style),
            right_aligned_text_x(frame.x, frame.width, runtime_width)
        );
    }

    #[test]
    fn retained_runtime_measure_style_uses_strong_face_for_heavy_text() {
        let mut style = style(UiTextAlign::Center, UiTextDirection::LeftToRight);
        style.font_weight = 650;
        let measure_style = retained_runtime_measure_style(&style);

        assert_eq!(
            measure_style.font_family.as_deref(),
            Some(runtime_font_family_for_face(HostTextFontFace::UiStrong))
        );
    }
}
