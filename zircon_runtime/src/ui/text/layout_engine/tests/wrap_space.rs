use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn word_wrap_trims_ascii_spaces_at_wrapped_line_edges() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("a", &style).width + 0.1;

    let layout = layout_text(
        "a b",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "a");
    assert_eq!(layout.lines[0].source_range.end, 1);
    assert_eq!(layout.lines[1].text, "b");
    assert_eq!(layout.lines[1].source_range.start, 2);
}

#[test]
fn word_wrap_preserves_non_breaking_space_at_paragraph_start() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);

    let layout = layout_text(
        "\u{00a0}b",
        &style,
        UiFrame::new(0.0, 0.0, 200.0, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "\u{00a0}b");
    assert_eq!(layout.lines[0].source_range.start, 0);
}
