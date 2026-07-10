use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextDirection, UiTextOverflow, UiTextRange, UiTextWrap},
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
