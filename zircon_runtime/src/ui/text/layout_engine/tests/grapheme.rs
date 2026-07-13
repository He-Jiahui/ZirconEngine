use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRichTextFormat, UiTextOverflow, UiTextWrap},
};

use super::{layout_text, test_style};

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
    style.rich_text_format = UiRichTextFormat::Markdown;

    let layout = layout_text(
        "*a*\u{0301}b",
        &style,
        UiFrame::new(0.0, 0.0, 5.0, 36.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "a\u{0301}");
    assert_eq!(layout.lines[0].runs.len(), 1);
    assert_eq!(layout.lines[0].runs[0].text, "a\u{0301}");
    assert_eq!(layout.lines[1].text, "b");
}
