use crate::core::framework::render::{RichTextFormat, StyleOverride};

use super::parse_rich_text;

#[test]
fn text_rich_bbcode_nested_styles_flatten_to_runs() {
    let parsed = parse_rich_text("[b]a[i]b[/i][/b]", RichTextFormat::BbCode);

    assert_eq!(parsed.text, "ab");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].byte_range, (0, 1));
    assert_eq!(parsed.runs[0].style.weight, Some(700));
    assert_eq!(parsed.runs[0].style.italic, None);
    assert_eq!(parsed.runs[1].byte_range, (1, 2));
    assert_eq!(parsed.runs[1].style.weight, Some(700));
    assert_eq!(parsed.runs[1].style.italic, Some(true));
}

#[test]
fn text_rich_color_size_font_overrides() {
    let parsed = parse_rich_text(
        "[color=#f00][size=24][font=Inter]red[/font][/size][/color] plain",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "red plain");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].style.font_size, Some(24.0));
    assert_eq!(
        parsed.runs[0]
            .style
            .family
            .as_ref()
            .map(|family| family.as_str()),
        Some("Inter")
    );
    assert_eq!(
        parsed.runs[0].style.color.unwrap().to_array(),
        [1.0, 0.0, 0.0, 1.0]
    );
    assert_eq!(parsed.runs[1].style, StyleOverride::default());
}

#[test]
fn text_rich_run_boundaries_respect_clusters() {
    let parsed = parse_rich_text("a[b]\u{0301}[/b]x", RichTextFormat::BbCode);

    assert_eq!(parsed.text, "a\u{0301}x");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].byte_range, (0, 4));
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
}

#[test]
fn text_rich_markdown_compat_unchanged() {
    let parsed = parse_rich_text("plain **bold** *italic* `code`", RichTextFormat::Markdown);

    assert_eq!(parsed.text, "plain bold italic code");
    assert_eq!(parsed.runs.len(), 6);
    assert_eq!(parsed.runs[1].style.weight, Some(700));
    assert_eq!(parsed.runs[3].style.italic, Some(true));
    assert_eq!(parsed.runs[5].style.code, Some(true));
}
