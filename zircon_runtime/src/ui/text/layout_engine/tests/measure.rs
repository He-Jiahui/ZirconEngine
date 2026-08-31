use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRichTextFormat, UiTextOverflow, UiTextWrap, UiTextWritingMode},
};

use super::super::{intrinsic_measurement_frame, measure_unwrapped_text_height};
use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_measurement_uses_backend_glyph_metrics() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let narrow = measure_text_size("iii", &style);
    let wide = measure_text_size("WWW", &style);
    let combined = measure_text_size("a\u{0301}b", &style);

    assert!(
        wide.width > narrow.width,
        "text measurement must use backend glyph metrics instead of a fixed grapheme advance"
    );
    assert!(combined.width < wide.width);
    assert_eq!(combined.height, 12.0);
}

#[test]
fn unwrapped_height_counts_unicode_breaks_and_a_trailing_empty_line() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let height = measure_unwrapped_text_height("a\r\nb\u{2028}", &style)
        .expect("plain unwrapped text has a fixed line height");

    assert_eq!(height, 36.0);
}

#[test]
fn unwrapped_height_matches_complete_measurement_across_hard_line_boundaries() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let budget = crate::text::TextShapingWorkBudget::default();
    let long_run = "a".repeat(budget.max_inline_input_bytes() + 1);
    assert!(budget.exceeds_inline_threshold(long_run.len()));

    for text in ["a\r\nb\u{2028}", long_run.as_str()] {
        let shortcut = measure_unwrapped_text_height(text, &style)
            .expect("plain unwrapped text has a fixed line height");
        let complete = measure_text_size(text, &style).height;

        assert!((shortcut - complete).abs() < 0.01, "text={text:?}");
    }
}

#[test]
fn unwrapped_fallback_height_is_certified_or_declined_without_a_latin_sample() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let shortcut = measure_unwrapped_text_height("世界", &style);
    let complete = measure_text_size("世界", &style).height;

    if let Some(shortcut) = shortcut {
        assert!(
            (shortcut - complete).abs() < 0.01,
            "a certified fallback chain must agree with complete measurement"
        );
    }
}

#[test]
fn text_measurement_uses_rich_run_metrics_instead_of_flat_base_style() {
    let mut rich_style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    rich_style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let plain_style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let rich = measure_text_size("[size=40]Wide[/size]", &rich_style);
    let plain = measure_text_size("Wide", &plain_style);

    assert!(rich.width > plain.width);
    assert!(rich.height > plain.height);
}

#[test]
fn intrinsic_measurement_frame_uses_unbounded_main_axis_without_byte_extent() {
    let mut rich_style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    rich_style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let rich_frame = intrinsic_measurement_frame("[b]text[/b]", &rich_style);

    assert_eq!(rich_frame.width, f32::INFINITY);
    assert_eq!(rich_frame.height, f32::INFINITY);

    let mut vertical_style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    vertical_style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let vertical_frame = intrinsic_measurement_frame("one\ntwo", &vertical_style);

    assert_eq!(vertical_frame.height, f32::INFINITY);
    assert_eq!(vertical_frame.width, vertical_style.line_height * 2.0);
}

#[test]
fn text_measurement_uses_vertical_layout_contract() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_writing_mode = UiTextWritingMode::VerticalRl;

    let measured = measure_text_size("ABCD", &style);
    let layout = layout_text(
        "ABCD",
        &style,
        UiFrame::new(0.0, 0.0, 1_000.0, 1_000.0),
        None,
    );

    assert!((measured.width - layout.measured_width).abs() < 0.1);
    assert!((measured.height - layout.measured_height).abs() < 0.1);
    assert!(measured.height > measured.width);
}

#[test]
fn text_layout_exports_backend_grapheme_advances() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let layout = layout_text("Wi", &style, UiFrame::new(0.0, 0.0, 200.0, 12.0), None);

    let line = &layout.lines[0];
    assert_eq!(line.glyph_advances.len(), 2);
    assert!(
        (line.glyph_advances.iter().sum::<f32>() - line.measured_width).abs() < 0.1,
        "resolved text line must export the same backend advances used for its measured width"
    );
    assert!(
        (line.glyph_advances[0] - line.glyph_advances[1]).abs() > 0.1,
        "per-grapheme advances must preserve backend width variation"
    );
}
