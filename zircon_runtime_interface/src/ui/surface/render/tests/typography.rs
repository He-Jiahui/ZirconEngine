use super::{resolve_ui_text_render_mode, UiRichTextFormat, UiTextRenderMode};

#[test]
fn rich_text_formats_use_versioned_wire_identity_and_reject_legacy_promises() {
    for (format, wire_value) in [
        (UiRichTextFormat::Plain, "plain"),
        (UiRichTextFormat::MarkdownInlineV1, "markdown_inline_v1"),
        (UiRichTextFormat::BbCodeV1, "bbcode_v1"),
        (UiRichTextFormat::HtmlSubsetV1, "html_subset_v1"),
    ] {
        let encoded = serde_json::to_string(&format).expect("format serializes");
        assert_eq!(encoded, format!("\"{wire_value}\""));
        assert_eq!(
            serde_json::from_str::<UiRichTextFormat>(&encoded).expect("format round trips"),
            format
        );
    }

    for legacy_value in ["markdown", "bbcode", "html"] {
        assert!(serde_json::from_str::<UiRichTextFormat>(&format!("\"{legacy_value}\"")).is_err());
    }
}

#[test]
fn text_render_mode_resolution_uses_explicit_request_then_font_default() {
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Native, Some(UiTextRenderMode::Sdf)),
        UiTextRenderMode::Native
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Sdf, Some(UiTextRenderMode::Native)),
        UiTextRenderMode::Sdf
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Sdf)),
        UiTextRenderMode::Sdf
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Auto, None),
        UiTextRenderMode::Native
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Msdf, Some(UiTextRenderMode::Native)),
        UiTextRenderMode::Msdf
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Mtsdf)),
        UiTextRenderMode::Mtsdf
    );
}
