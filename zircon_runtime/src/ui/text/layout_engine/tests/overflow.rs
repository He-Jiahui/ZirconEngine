use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextAlign, UiTextOverflow, UiTextRange, UiTextWrap},
};

use super::{ellipsis_width_for_test, layout_text, measure_text_size, test_style};

#[test]
fn ellipsis_preserves_combining_mark_grapheme_boundaries() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);

    let layout = layout_text(
        "a\u{0301}bc",
        &style,
        UiFrame::new(0.0, 0.0, ellipsis_width_for_test(&style), 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "a\u{0301}…");
    assert_eq!(layout.lines[0].runs[0].source_range.end, "a\u{0301}".len());
}

#[test]
fn ellipsis_preserves_rich_run_boundary_grapheme_clusters() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    style.rich_text = true;

    let layout = layout_text(
        "*a*\u{0301}bc",
        &style,
        UiFrame::new(0.0, 0.0, ellipsis_width_for_test(&style), 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "a\u{0301}…");
    assert_eq!(layout.lines[0].runs.len(), 3);
    assert_eq!(layout.lines[0].runs[0].text, "a");
    assert_eq!(layout.lines[0].runs[1].text, "\u{0301}");
    assert_eq!(layout.lines[0].runs[2].text, "…");
}

#[test]
fn end_ellipsis_keeps_head_graphemes_through_overflow_owner() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    let frame_width = measure_text_size("a…", &style).width + 0.1;

    let layout = layout_text(
        "abcdef",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "a…");
    assert_eq!(layout.lines[0].runs.len(), 2);
    assert_eq!(layout.lines[0].runs[0].text, "a");
    assert_eq!(
        layout.lines[0].runs[0].source_range,
        UiTextRange { start: 0, end: 1 }
    );
    assert_eq!(layout.lines[0].runs[1].text, "…");
}

#[test]
fn word_ellipsis_trims_partial_word_before_marker() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::EllipsisWord);
    let frame_width = measure_text_size("alpha b…", &style).width + 0.1;

    let layout = layout_text(
        "alpha beta gamma",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "alpha…");
    assert_eq!(layout.lines[0].runs.len(), 2);
    assert_eq!(layout.lines[0].runs[0].text, "alpha");
    assert_eq!(
        layout.lines[0].runs[0].source_range,
        UiTextRange { start: 0, end: 5 }
    );
    assert_eq!(layout.lines[0].runs[1].text, "…");
    assert!(layout.lines[0].measured_width <= frame_width + 0.1);
}

#[test]
fn word_ellipsis_drops_partial_first_word() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::EllipsisWord);
    let frame_width = measure_text_size("alp…", &style).width + 0.1;

    let layout = layout_text(
        "alpha beta",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "…");
    assert_eq!(layout.lines[0].runs.len(), 1);
    assert_eq!(layout.lines[0].runs[0].text, "…");
}

#[test]
fn start_ellipsis_keeps_tail_graphemes() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::EllipsisStart);
    let frame_width = measure_text_size("…f", &style).width + 0.1;

    let layout = layout_text(
        "abcdef",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "…f");
    assert_eq!(layout.lines[0].runs.len(), 2);
    assert_eq!(layout.lines[0].runs[0].text, "…");
    assert_eq!(
        layout.lines[0].runs[0].source_range,
        UiTextRange { start: 0, end: 0 }
    );
    assert_eq!(layout.lines[0].runs[1].text, "f");
    assert_eq!(
        layout.lines[0].runs[1].source_range,
        UiTextRange { start: 5, end: 6 }
    );
    assert!(layout.lines[0].measured_width <= frame_width + 0.1);
}

#[test]
fn middle_ellipsis_keeps_head_and_tail_graphemes() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::EllipsisMiddle);
    let frame_width = measure_text_size("a…f", &style).width + 0.1;

    let layout = layout_text(
        "abcdef",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "a…f");
    assert_eq!(layout.lines[0].runs.len(), 3);
    assert_eq!(layout.lines[0].runs[0].text, "a");
    assert_eq!(
        layout.lines[0].runs[0].source_range,
        UiTextRange { start: 0, end: 1 }
    );
    assert_eq!(layout.lines[0].runs[1].text, "…");
    assert_eq!(
        layout.lines[0].runs[1].source_range,
        UiTextRange { start: 6, end: 6 }
    );
    assert_eq!(layout.lines[0].runs[2].text, "f");
    assert_eq!(
        layout.lines[0].runs[2].source_range,
        UiTextRange { start: 5, end: 6 }
    );
    assert!(layout.lines[0].measured_width <= frame_width + 0.1);
}

#[test]
fn horizontal_ellipsis_applies_to_nowrap_overwide_line() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Ellipsis);
    let frame_width = measure_text_size("ab…", &style).width + 0.1;

    let layout = layout_text(
        "abcdef",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 24.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.overflow_clipped);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "ab…");
    assert!(layout.lines[0].measured_width <= frame_width + 0.1);
}

#[test]
fn horizontal_start_ellipsis_non_last_line_is_not_justified() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::EllipsisStart);
    style.text_align = UiTextAlign::Justify;
    let frame_width = measure_text_size("… beta gamma", &style).width + 0.1;

    let layout = layout_text(
        "alpha beta gamma\nz",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 36.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert!(layout.overflow_clipped);
    assert!(layout.lines[0].ellipsized);
    assert_eq!(layout.lines[0].text, "… beta gamma");
    assert!(
        layout.lines[0].frame.width < frame_width - 0.01,
        "ellipsized non-last lines must not be justified back to the full frame"
    );
}
