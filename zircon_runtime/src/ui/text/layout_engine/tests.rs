use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRange, UiTextWrap,
    },
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
fn text_bidi_mirrors_paren_in_rtl() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::RightToLeft;

    let layout = layout_text(
        "שלום (אב)",
        &style,
        UiFrame::new(0.0, 0.0, 160.0, 12.0),
        None,
    );
    let line = &layout.lines[0];

    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert_eq!(line.text, "(בא) םולש");
    assert!(line
        .runs
        .iter()
        .any(|run| run.text == "(" && run.source_range == UiTextRange { start: 14, end: 15 }));
    assert!(line
        .runs
        .iter()
        .any(|run| run.text == ")" && run.source_range == UiTextRange { start: 9, end: 10 }));
}

#[test]
fn text_bidi_mirrors_arrow_in_rtl() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::RightToLeft;

    let layout = layout_text("אב →", &style, UiFrame::new(0.0, 0.0, 160.0, 12.0), None);
    let line = &layout.lines[0];

    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert_eq!(line.text, "← בא");
    assert!(line
        .runs
        .iter()
        .any(|run| run.text == "←" && run.source_range == UiTextRange { start: 5, end: 8 }));
}

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

#[test]
fn word_wrap_uses_uax14_cjk_break_opportunities() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);

    let layout = layout_text("中文", &style, UiFrame::new(0.0, 0.0, 12.0, 36.0), None);

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "文");
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);

    let layout = layout_text("中文。", &style, UiFrame::new(0.0, 0.0, 12.0, 48.0), None);

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "文。");
    assert!(
        layout.lines.iter().all(|line| !line.text.starts_with('。')),
        "CJK kinsoku must prevent forbidden punctuation from starting a wrapped line"
    );
}

#[test]
fn text_wrap_soft_hyphen_inserts_hyphen() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("pre-", &style).width + 0.1;
    assert!(frame_width < measure_text_size("prefix", &style).width);
    assert!(measure_text_size("fix", &style).width <= frame_width);

    let layout = layout_text(
        "pre\u{00ad}fix",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "pre-");
    assert_eq!(layout.lines[1].text, "fix");
    assert!(
        layout
            .lines
            .iter()
            .all(|line| !line.text.contains('\u{00ad}')),
        "soft hyphen is a source break hint and must not be retained in visual text"
    );
}

#[test]
fn text_wrap_long_word_falls_back_to_glyph() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("a", &style).width + 0.1;

    let layout = layout_text(
        "abcd",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 72.0),
        None,
    );

    assert_eq!(layout.lines.len(), 4);
    assert_eq!(layout.lines[0].text, "a");
    assert_eq!(layout.lines[1].text, "b");
    assert_eq!(layout.lines[2].text, "c");
    assert_eq!(layout.lines[3].text, "d");
}

#[test]
fn word_wrap_keeps_non_breaking_space_group_together() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("a", &style).width + 0.1;

    let layout = layout_text(
        "a\u{00a0}b",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "a\u{00a0}b");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "NBSP is glue: the unbreakable group may overhang instead of being split by glyph fallback"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_trailing_open_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("中", &style).width + 0.1;

    let layout = layout_text(
        "中（文",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "（文");
    assert!(
        layout.lines.iter().all(|line| !line.text.ends_with('（')),
        "CJK kinsoku must prevent opening punctuation from ending a wrapped line"
    );
}

#[test]
fn text_align_start_end_follow_rtl_base_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::RightToLeft;
    let frame = UiFrame::new(10.0, 0.0, 80.0, 12.0);

    style.text_align = UiTextAlign::Start;
    let start_layout = layout_text("שלום", &style, frame, None);
    let start_line = &start_layout.lines[0];
    assert_eq!(start_layout.direction, UiTextDirection::RightToLeft);
    assert!(
        (start_line.frame.right() - frame.right()).abs() < 0.01,
        "RTL start alignment must anchor text to the right edge"
    );

    style.text_align = UiTextAlign::End;
    let end_layout = layout_text("שלום", &style, frame, None);
    assert!(
        (end_layout.lines[0].frame.x - frame.x).abs() < 0.01,
        "RTL end alignment must anchor text to the left edge"
    );
}

#[test]
fn text_align_start_end_auto_uses_first_strong_rtl_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Auto;
    let frame = UiFrame::new(10.0, 0.0, 120.0, 12.0);

    style.text_align = UiTextAlign::Start;
    let start_layout = layout_text("שלום abc", &style, frame, None);
    let start_line = &start_layout.lines[0];
    assert_eq!(start_layout.direction, UiTextDirection::RightToLeft);
    assert!((start_line.frame.right() - frame.right()).abs() < 0.01);

    style.text_align = UiTextAlign::End;
    let end_layout = layout_text("שלום abc", &style, frame, None);
    assert_eq!(end_layout.direction, UiTextDirection::RightToLeft);
    assert!((end_layout.lines[0].frame.x - frame.x).abs() < 0.01);
}

#[test]
fn text_align_start_end_auto_uses_first_strong_ltr_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Auto;
    let frame = UiFrame::new(10.0, 0.0, 120.0, 12.0);

    style.text_align = UiTextAlign::Start;
    let start_layout = layout_text("abc שלום", &style, frame, None);
    assert_eq!(start_layout.direction, UiTextDirection::LeftToRight);
    assert!((start_layout.lines[0].frame.x - frame.x).abs() < 0.01);

    style.text_align = UiTextAlign::End;
    let end_layout = layout_text("abc שלום", &style, frame, None);
    assert_eq!(end_layout.direction, UiTextDirection::LeftToRight);
    assert!((end_layout.lines[0].frame.right() - frame.right()).abs() < 0.01);
}

#[test]
fn text_align_start_end_mixed_request_uses_first_strong_base_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Mixed;
    style.text_align = UiTextAlign::Start;
    let frame = UiFrame::new(10.0, 0.0, 120.0, 12.0);

    let layout = layout_text("שלום abc", &style, frame, None);

    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert!((layout.lines[0].frame.right() - frame.right()).abs() < 0.01);
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

fn ellipsis_width_for_test(style: &UiResolvedStyle) -> f32 {
    let minimum = measure_text_size("a\u{0301}…", style).width + 0.1;
    let maximum = measure_text_size("a\u{0301}b…", style).width - 0.1;
    minimum
        .min(maximum)
        .max(measure_text_size("…", style).width)
}
