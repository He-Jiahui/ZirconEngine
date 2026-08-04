use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRichTextFormat, UiTextOverflow, UiTextWrap, UiTextWritingMode},
};

use super::{layout_text, measure_text_size, test_style};

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
fn rich_inline_word_wrap_projects_soft_hyphen_suffix_into_visual_line() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;
    let frame_width = measure_text_size("pre-", &style).width + 0.1;

    let layout = layout_text(
        "pre\u{00ad}fix<img src=\"res://icons/star.png\" width=\"1\" height=\"12\">",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 60.0),
        None,
    );

    assert!(layout.lines.len() >= 2);
    assert_eq!(layout.lines[0].text, "pre-");
    assert!(layout
        .lines
        .iter()
        .all(|line| !line.text.contains('\u{00ad}')));
}

#[test]
fn rich_inline_vertical_word_wrap_projects_soft_hyphen_suffix_into_column() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;
    let column_height = measure_text_size("pre-", &style).width + 0.1;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;

    let layout = layout_text(
        "pre\u{00ad}fix<img src=\"res://icons/star.png\" width=\"1\" height=\"12\">",
        &style,
        UiFrame::new(0.0, 0.0, 60.0, column_height),
        None,
    );

    assert!(layout.lines.len() >= 2);
    assert_eq!(layout.lines[0].text, "pre-");
    assert!(layout
        .lines
        .iter()
        .all(|line| !line.text.contains('\u{00ad}')));
}
