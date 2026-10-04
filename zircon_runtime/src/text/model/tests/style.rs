use super::{RichTextFormat, TextStyle};

#[test]
fn rich_text_formats_use_versioned_artifact_identity() {
    for (format, wire_value) in [
        (RichTextFormat::Plain, "plain"),
        (RichTextFormat::MarkdownInlineV1, "markdown_inline_v1"),
        (RichTextFormat::BbCodeV1, "bbcode_v1"),
        (RichTextFormat::HtmlSubsetV1, "html_subset_v1"),
    ] {
        let encoded = serde_json::to_string(&format).expect("format serializes");
        assert_eq!(encoded, format!("\"{wire_value}\""));
        assert_eq!(
            serde_json::from_str::<RichTextFormat>(&encoded).expect("format round trips"),
            format
        );
    }

    for legacy_value in ["markdown", "bbcode", "html"] {
        assert!(serde_json::from_str::<RichTextFormat>(&format!("\"{legacy_value}\"")).is_err());
    }
}

#[test]
fn legacy_text_style_defaults_new_shaping_identity_fields() {
    let legacy = r#"{
            "font": null,
            "font_family": null,
            "language": null,
            "font_weight": 400,
            "font_size": 16.0,
            "line_height": 19.2,
            "tab_size": 4.0,
            "text_align": "left",
            "wrap": "word"
        }"#;

    let style: TextStyle = serde_json::from_str(legacy).expect("legacy style remains readable");

    assert!(!style.italic);
    assert!(style.features.is_empty());
}
