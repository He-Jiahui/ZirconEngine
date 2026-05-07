use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size};

#[test]
fn glyph_wrap_preserves_combining_mark_grapheme_boundaries() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);

    let layout = layout_text(
        "a\u{0301}bc",
        &style,
        UiFrame::new(0.0, 0.0, 5.0, 36.0),
        None,
    );

    assert_eq!(layout.lines.len(), 3);
    assert_eq!(layout.lines[0].text, "a\u{0301}");
    assert_eq!(layout.lines[0].source_range.start, 0);
    assert_eq!(layout.lines[0].source_range.end, "a\u{0301}".len());
    assert_eq!(layout.lines[1].text, "b");
    assert_eq!(layout.lines[2].text, "c");
}

#[test]
fn glyph_wrap_preserves_rich_run_boundary_grapheme_clusters() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text = true;

    let layout = layout_text(
        "*a*\u{0301}b",
        &style,
        UiFrame::new(0.0, 0.0, 5.0, 36.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "a\u{0301}");
    assert_eq!(layout.lines[0].runs.len(), 2);
    assert_eq!(layout.lines[0].runs[0].text, "a");
    assert_eq!(layout.lines[0].runs[1].text, "\u{0301}");
    assert_eq!(layout.lines[1].text, "b");
}

#[test]
fn ellipsis_preserves_combining_mark_grapheme_boundaries() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);

    let layout = layout_text(
        "a\u{0301}bc",
        &style,
        UiFrame::new(0.0, 0.0, 10.0, 12.0),
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
        UiFrame::new(0.0, 0.0, 10.0, 12.0),
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
fn rtl_visual_order_reverses_grapheme_clusters() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let layout = layout_text(
        "abc ש\u{05b8}ל def",
        &style,
        UiFrame::new(0.0, 0.0, 120.0, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "abc לש\u{05b8} def");
}

#[test]
fn rtl_visual_order_preserves_rich_run_boundary_grapheme_clusters() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text = true;

    let layout = layout_text(
        "abc *ש*\u{05b8}ל def",
        &style,
        UiFrame::new(0.0, 0.0, 120.0, 12.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "abc לש\u{05b8} def");
    let run_texts: Vec<_> = layout.lines[0]
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    assert_eq!(run_texts, vec!["abc ", "ל", "ש", "\u{05b8}", " def"]);
}

#[test]
fn text_measurement_counts_grapheme_clusters() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    assert_eq!(
        measure_text_size("a\u{0301}b", &style),
        UiSize::new(10.0, 12.0)
    );
}

fn test_style(wrap: UiTextWrap, overflow: UiTextOverflow) -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap,
        text_overflow: overflow,
        ..UiResolvedStyle::default()
    }
}
