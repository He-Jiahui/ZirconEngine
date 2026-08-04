use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRichTextFormat, UiTextDirection, UiTextOverflow, UiTextRange, UiTextWrap},
};

use super::{layout_text, test_style};

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
    style.rich_text_format = UiRichTextFormat::Markdown;

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
    assert_eq!(run_texts, vec!["abc ", "ל", "ש\u{05b8}", " def"]);
}

#[test]
fn auto_direction_uses_uax9_cjk_strong_l_before_hebrew() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Auto;

    let layout = layout_text("中文 אב", &style, UiFrame::new(0.0, 0.0, 160.0, 12.0), None);

    assert_eq!(layout.direction, UiTextDirection::LeftToRight);
    assert_eq!(layout.lines[0].text, "中文 בא");
}

#[test]
fn bidi_isolate_keeps_outer_ltr_order_and_reorders_inner_rtl() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Auto;
    let text = "A \u{2067}אב\u{2069} Z";

    let layout = layout_text(text, &style, UiFrame::new(0.0, 0.0, 160.0, 12.0), None);
    let visible_text = layout.lines[0]
        .text
        .chars()
        .filter(|ch| !matches!(*ch, '\u{2066}' | '\u{2067}' | '\u{2068}' | '\u{2069}'))
        .collect::<String>();

    assert_eq!(layout.direction, UiTextDirection::LeftToRight);
    assert_eq!(visible_text, "A בא Z");
    assert!(layout.lines[0]
        .runs
        .iter()
        .any(|run| run.text == "ב" && run.direction == UiTextDirection::RightToLeft));
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
fn text_bidi_mirrors_unicode_math_bracket_in_rtl() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::RightToLeft;

    let layout = layout_text(
        "\u{05D0}\u{05D1} \u{27E8}",
        &style,
        UiFrame::new(0.0, 0.0, 160.0, 12.0),
        None,
    );

    assert_eq!(layout.lines[0].text, "\u{27E9} \u{05D1}\u{05D0}");
}

#[test]
fn bidi_visual_order_keeps_grapheme_advances_in_visual_sequence() {
    use super::super::candidate_line::{append_segment, CandidateLine};
    use super::super::visual_order::apply_visual_order_with_advances;
    use zircon_runtime_interface::ui::surface::UiTextRunKind;

    let text = "A אב";
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        text,
        UiTextRange {
            start: 0,
            end: text.len(),
        },
    );
    let logical_advances = vec![1.0, 2.0, 3.0, 4.0];
    let mut visual_advances = logical_advances.clone();

    apply_visual_order_with_advances(
        &mut line,
        text,
        UiTextDirection::LeftToRight,
        &mut visual_advances,
    );

    assert_eq!(line.text, "A בא");
    assert_eq!(visual_advances, vec![1.0, 2.0, 4.0, 3.0]);
}

#[test]
fn bidi_visual_order_keeps_virtual_tatweel_in_rtl_visual_sequence() {
    use super::super::candidate_line::{CandidateLine, append_segment, insert_virtual_text};
    use super::super::visual_order::apply_visual_order_with_advances;
    use zircon_runtime_interface::ui::surface::UiTextRunKind;

    let source = "\u{0633}\u{0644}\u{0627}\u{0645}";
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        source,
        UiTextRange {
            start: 0,
            end: source.len(),
        },
    );
    assert!(insert_virtual_text(&mut line, 2, "\u{0640}"));
    assert!(insert_virtual_text(&mut line, 6, "\u{0640}"));
    let mut visual_advances = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    apply_visual_order_with_advances(
        &mut line,
        source,
        UiTextDirection::RightToLeft,
        &mut visual_advances,
    );

    assert_eq!(
        line.text,
        "\u{0645}\u{0627}\u{0640}\u{0644}\u{0640}\u{0633}"
    );
    assert_eq!(visual_advances, vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]);
    assert_eq!(
        line.runs
            .iter()
            .filter(|run| run.source_range.start == run.source_range.end)
            .map(|run| run.source_range.start)
            .collect::<Vec<_>>(),
        vec![4, 2]
    );
}

#[test]
fn bidi_visual_order_keeps_a_grapheme_split_across_style_runs_atomic() {
    use super::super::candidate_line::{append_segment, CandidateLine};
    use super::super::visual_order::apply_visual_order_with_advances;
    use zircon_runtime_interface::ui::surface::UiTextRunKind;

    let text = "a\u{301} אב";
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "a",
        UiTextRange { start: 0, end: 1 },
    );
    append_segment(
        &mut line,
        UiTextRunKind::Strong,
        "\u{301} אב",
        UiTextRange {
            start: 1,
            end: text.len(),
        },
    );
    let mut visual_advances = vec![2.0, 1.0, 3.0, 4.0];

    apply_visual_order_with_advances(
        &mut line,
        text,
        UiTextDirection::LeftToRight,
        &mut visual_advances,
    );

    assert_eq!(line.text, "a\u{301} בא");
    assert_eq!(visual_advances, vec![2.0, 1.0, 4.0, 3.0]);
    assert_eq!(
        line.runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        line.text
    );
}
