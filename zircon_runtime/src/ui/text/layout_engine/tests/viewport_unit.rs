use super::{
    line_window, visible_plain_text_lines, visible_plain_text_lines_from_certified_height,
};
use crate::{
    text::{RichTextFormat, SharedTextLayoutSession, TextDocumentKey},
    ui::text::rich_text::parse_source_text as try_parse_source_text,
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextOverflow, UiTextRange, UiTextWrap, UiTextWritingMode,
};

fn parse_source_text(
    text: &str,
    format: RichTextFormat,
) -> crate::ui::text::rich_text::UiParsedText {
    try_parse_source_text(text, format).expect("test text fits parser budgets")
}

#[test]
fn line_window_keeps_two_viewports_of_overscan() {
    assert_eq!(line_window(120.0, 20.0, 10.0, 100, 2), Some((8, 16)));
}

#[test]
fn line_window_clamps_at_document_edges() {
    assert_eq!(line_window(0.0, 20.0, 10.0, 5, 2), Some((0, 5)));
    assert_eq!(line_window(1_000.0, 20.0, 10.0, 5, 2), Some((1, 5)));
}

#[test]
fn unbounded_line_window_saturates_before_document_count_is_known() {
    assert_eq!(
        super::unbounded_line_window(f32::MAX, 20.0, 10.0, 2),
        Some(usize::MAX..usize::MAX)
    );
}

#[test]
fn visible_window_rejects_vertical_text() {
    let parsed = parse_source_text("vertical", RichTextFormat::Plain);
    let style = UiResolvedStyle {
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Clip,
        text_writing_mode: UiTextWritingMode::VerticalRl,
        ..UiResolvedStyle::default()
    };
    let viewport =
        super::super::UiTextViewport::new(0.0, 20.0, 2).expect("finite document viewport");
    let mut provider = SharedTextLayoutSession::new();

    assert!(
        visible_plain_text_lines(&parsed, &style, viewport, 10.0, None, &mut provider).is_none()
    );
}

#[test]
fn visible_window_requires_a_certified_uniform_line_height() {
    let parsed = parse_source_text("first\nsecond\nthird", RichTextFormat::Plain);
    let viewport =
        super::super::UiTextViewport::new(11.0, 1.0, 0).expect("finite document viewport");
    let mut provider = SharedTextLayoutSession::new();

    assert!(
        visible_plain_text_lines_from_certified_height(
            &parsed,
            viewport,
            None,
            None,
            &mut provider,
        )
        .is_none(),
        "an uncertified font chain must use complete physical-line layout"
    );
}

#[test]
fn visible_window_borrows_only_the_requested_plain_hard_lines() {
    let parsed = parse_source_text("first\nsecond\nthird\nfourth", RichTextFormat::Plain);
    let viewport =
        super::super::UiTextViewport::new(11.0, 1.0, 0).expect("finite document viewport");
    let mut provider = SharedTextLayoutSession::new();

    let window = visible_plain_text_lines_from_certified_height(
        &parsed,
        viewport,
        Some(10.0),
        None,
        &mut provider,
    )
    .expect("partial plain viewport window");

    assert_eq!(window.first_line, 1);
    assert_eq!(window.total_line_count, 4);
    assert_eq!(
        window
            .lines
            .iter()
            .map(|line| (line.text.as_str(), line.source_range))
            .collect::<Vec<_>>(),
        vec![("second", UiTextRange { start: 6, end: 12 })]
    );
}

#[test]
fn visible_windows_reuse_the_session_hard_line_index() {
    let parsed = parse_source_text("zero\none\ntwo\nthree", RichTextFormat::Plain);
    let mut provider = SharedTextLayoutSession::new();
    let first = super::super::UiTextViewport::new(0.0, 1.0, 0).expect("finite document viewport");
    let third = super::super::UiTextViewport::new(21.0, 1.0, 0).expect("finite document viewport");

    let key = TextDocumentKey::new(7, 1);
    let first_window = visible_plain_text_lines_from_certified_height(
        &parsed,
        first,
        Some(10.0),
        Some(key),
        &mut provider,
    )
    .expect("first plain viewport window");
    let third_window = visible_plain_text_lines_from_certified_height(
        &parsed,
        third,
        Some(10.0),
        Some(key),
        &mut provider,
    )
    .expect("third plain viewport window");

    assert_eq!(first_window.lines[0].text, "zero");
    assert_eq!(third_window.lines[0].text, "two");
    let report = provider.hard_line_index_report();
    assert_eq!(report.build_count, 1);
    assert_eq!(report.hit_count, 1);
}
