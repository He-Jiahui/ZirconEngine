use super::super::super::super::super::paint_text::{
    runtime_font_family_for_face, HostTextFontFace,
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
    let runtime_family = runtime_font_family_for_face(HostTextFontFace::Ui);

    assert_eq!(
        measure_style.font_family.as_deref(),
        Some(runtime_family.as_ref())
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
    let runtime_family = runtime_font_family_for_face(HostTextFontFace::UiStrong);

    assert_eq!(
        measure_style.font_family.as_deref(),
        Some(runtime_family.as_ref())
    );
}
