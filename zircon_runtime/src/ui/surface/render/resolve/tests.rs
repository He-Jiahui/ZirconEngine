use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiRichTextFormat, UiTextWritingMode};

use super::*;

#[test]
fn resolve_style_parses_text_tab_size_alias() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
text_tab_size = 6.0
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.tab_size, 6.0);
}

#[test]
fn resolve_style_projects_valid_line_height_ratio_from_logical_font_size() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
font_size = 20.0
line_height_ratio = 1.5
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.font_size, 20.0);
    assert_eq!(style.line_height, 30.0);
}

#[test]
fn resolve_style_rejects_nonpositive_line_height_ratio() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
font_size = 20.0
line_height_ratio = 0.0
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    assert_eq!(
        resolve_style(Some(&metadata)).line_height,
        UiResolvedStyle::default_line_height(20.0)
    );
}

#[test]
fn resolve_style_parses_and_clamps_font_weight_aliases() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
[font]
weight = 620.4
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.font_weight, 620);

    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
text_font_weight = 1800.0
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.font_weight, UiResolvedStyle::MAX_FONT_WEIGHT);
}

#[test]
fn resolve_style_parses_text_writing_mode_alias() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
writing_mode = "vertical-rl"
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.text_writing_mode, UiTextWritingMode::VerticalRl);
}

#[test]
fn resolve_style_parses_run_language_from_font_table() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
[font]
language = "zh-Hans-CN"
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.language.as_deref(), Some("zh-Hans-CN"));
}

#[test]
fn resolve_style_preserves_invalid_language_for_typed_shaping_rejection() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
[font]
language = "en--US"
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    let style = resolve_style(Some(&metadata));

    assert_eq!(style.language.as_deref(), Some("en--US"));
}

#[test]
fn resolve_style_selects_explicit_rich_text_format() {
    let metadata = UiTemplateNodeMetadata {
        attributes: toml::from_str(
            r#"
rich_text_format = "html_subset_v1"
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    assert_eq!(
        resolve_style(Some(&metadata)).rich_text_format,
        UiRichTextFormat::HtmlSubsetV1
    );
    assert_eq!(
        resolve_style(None).rich_text_format,
        UiRichTextFormat::Plain
    );
}

#[test]
fn autocomplete_render_and_editable_state_share_query_property() {
    let metadata = UiTemplateNodeMetadata {
        component: "Autocomplete".to_string(),
        attributes: toml::from_str(
            r#"
query = "needle"
value = "asset://selected"
caret_offset = 6
"#,
        )
        .unwrap(),
        ..Default::default()
    };

    assert_eq!(resolve_text(Some(&metadata)).as_deref(), Some("needle"));
    let editable = resolve_editable_text_state(Some(&metadata), Some("needle"))
        .expect("Autocomplete query should resolve as editable text");
    assert_eq!(editable.text, "needle");
    assert_eq!(editable.caret.offset, 6);
}
