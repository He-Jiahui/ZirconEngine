use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLayout, UiRichTextFormat, UiTextOverflow, UiTextWrap, UiTextWritingMode,
    },
};

use super::{layout_text, test_style};

fn assert_split_i_graphemes(layout: &UiResolvedTextLayout) {
    let line_text = layout
        .lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(line_text, ["i", "i"]);
}

#[test]
fn text_layout_wraps_at_a_narrow_horizontal_frame_width() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    let layout = layout_text("ii", &style, UiFrame::new(0.0, 0.0, 0.25, 48.0), None);

    assert_split_i_graphemes(&layout);
    assert!(layout.lines.iter().all(|line| line.frame.width <= 0.25));
}

#[test]
fn text_layout_keeps_an_unbounded_horizontal_frame_on_one_line() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    let layout = layout_text(
        "ii",
        &style,
        UiFrame::new(0.0, 0.0, f32::INFINITY, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "ii");
}

#[test]
fn text_layout_fails_closed_for_invalid_horizontal_wrap_extents() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    for width in [-1.0, f32::NAN] {
        let layout = layout_text("ii", &style, UiFrame::new(0.0, 0.0, width, 48.0), None);

        assert!(layout.lines.is_empty());
        assert!(layout.overflow_clipped);
    }
}

#[test]
fn text_layout_wraps_vertical_columns_at_a_narrow_frame_height() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let layout = layout_text("ii", &style, UiFrame::new(0.0, 0.0, 48.0, 0.25), None);

    assert_split_i_graphemes(&layout);
    assert!(layout.lines.iter().all(|line| line.frame.height <= 0.25));
}

#[test]
fn text_block_layout_preserves_a_narrow_horizontal_extent_after_indent() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCode;
    let layout = layout_text(
        "[p indent=12]ii[/p]",
        &style,
        UiFrame::new(0.0, 0.0, 0.25, 48.0),
        None,
    );

    assert_split_i_graphemes(&layout);
    assert!(layout.lines.iter().all(|line| line.frame.width <= 0.25));
}

#[test]
fn text_block_layout_preserves_a_narrow_vertical_extent_after_indent() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCode;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let layout = layout_text(
        "[p indent=12]ii[/p]",
        &style,
        UiFrame::new(0.0, 0.0, 48.0, 0.25),
        None,
    );

    assert_split_i_graphemes(&layout);
    assert!(layout.lines.iter().all(|line| line.frame.height <= 0.25));
}
