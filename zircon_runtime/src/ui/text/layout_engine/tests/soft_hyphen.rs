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
    assert_eq!(layout.lines[0].source_range.end, "pre\u{00ad}".len());
    assert!(layout.lines[0].runs.iter().any(|run| {
        run.text == "-"
            && run.source_range.start == "pre\u{00ad}".len()
            && run.source_range.start == run.source_range.end
    }));
    assert!(
        layout.rich_text_artifact.is_some(),
        "plain soft-hyphen output must retain its canonical logical-virtual glyph artifact"
    );
    assert!(
        layout
            .lines
            .iter()
            .all(|line| !line.text.contains('\u{00ad}')),
        "soft hyphen is a source break hint and must not be retained in visual text"
    );
}

#[test]
fn rich_layout_word_wrap_projects_soft_hyphen_suffix_into_visual_line() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::HtmlSubsetV1;
    let frame_width = measure_text_size("pre-", &style).width + 0.1;

    let layout = layout_text(
        "pre\u{00ad}fix<img src=\"res://icons/star.png\" width=\"1\" height=\"12\">",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 60.0),
        None,
    );

    assert!(layout.lines.len() >= 2);
    assert_eq!(layout.lines[0].text, "pre-");
    assert!(layout.lines[0].runs.iter().any(|run| {
        run.text == "-"
            && run.source_range.start == "pre\u{00ad}".len()
            && run.source_range.start == run.source_range.end
    }));
    let artifact = crate::text::resolve_resolved_text_glyph_artifact(
        layout
            .rich_text_artifact
            .as_ref()
            .expect("rich soft-hyphen artifact handle"),
    )
    .expect("rich soft-hyphen glyph artifact");
    assert!(artifact.lines[0].as_ref().is_some_and(|line| {
        line.glyphs.iter().any(|glyph| {
            glyph.flags.virtual_glyph
                && glyph.source_range.start == "pre\u{00ad}".len()
                && glyph.source_range.start == glyph.source_range.end
        })
    }));
    assert!(
        layout
            .lines
            .iter()
            .all(|line| !line.text.contains('\u{00ad}'))
    );
}

#[test]
fn rich_layout_vertical_word_wrap_projects_soft_hyphen_suffix_into_column() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::HtmlSubsetV1;
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
    assert!(layout.lines[0].runs.iter().any(|run| {
        run.text == "-"
            && run.source_range.start == "pre\u{00ad}".len()
            && run.source_range.end == "pre\u{00ad}".len()
    }));
    assert!(
        layout
            .lines
            .iter()
            .all(|line| !line.text.contains('\u{00ad}'))
    );
    let artifact = crate::text::resolve_resolved_text_glyph_artifact(
        layout
            .rich_text_artifact
            .as_ref()
            .expect("vertical soft hyphen keeps a composite artifact"),
    )
    .expect("vertical soft hyphen keeps the canonical glyph artifact");
    assert!(
        artifact.lines[0]
            .as_ref()
            .is_some_and(|line| line.glyphs.iter().any(|glyph| glyph.flags.virtual_glyph)),
        "the display-owned vertical soft hyphen must be a canonical virtual glyph"
    );
}
