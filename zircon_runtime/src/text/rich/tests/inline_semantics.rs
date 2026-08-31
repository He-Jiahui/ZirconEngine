use crate::text::{InlineBaseline, InlineObjectRef, RichParseResult, RichTextFormat};

use super::parser_registry::parse_rich_text as try_parse_rich_text;

fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    try_parse_rich_text(markup, format).expect("test rich source fits parser budgets")
}

#[test]
fn text_rich_inline_image_parses_placeholder_metric_contract() {
    let parsed = parse_rich_text(
        "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\" baseline=\"baseline\" alt=\"Favorite\" title=\"Favorite icon\">after",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "before\u{fffc}after");
    let image_run = parsed
        .runs
        .iter()
        .find(|run| run.inline.is_some())
        .expect("inline image run");
    assert_eq!(image_run.byte_range, (6, 9));
    assert!(matches!(
        image_run.inline.as_ref(),
        Some(InlineObjectRef::Image {
            size,
            baseline: InlineBaseline::Baseline,
            alternative_text: Some(alternative_text),
            tooltip: Some(tooltip),
            ..
        }) if size.to_array() == [16.0, 24.0]
            && alternative_text == "Favorite"
            && tooltip == "Favorite icon"
    ));
}

#[test]
fn text_rich_bbcode_image_attribute_form_retains_semantic_fallbacks() {
    let parsed = parse_rich_text(
        "[img src=\"res://icons/star.png\" alt=\"Favorite\" title=\"Favorite icon\"]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "\u{fffc}");
    assert!(matches!(
        parsed.runs[0].inline.as_ref(),
        Some(InlineObjectRef::Image {
            alternative_text: Some(alternative_text),
            tooltip: Some(tooltip),
            ..
        }) if alternative_text == "Favorite" && tooltip == "Favorite icon"
    ));
}

#[test]
fn text_rich_hyperlink_carries_typed_target_and_hit_range() {
    let parsed = parse_rich_text(
        "go <a href=\"res://docs/help\">help</a> now",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "go help now");
    let link = parsed
        .runs
        .iter()
        .find(|run| run.link.is_some())
        .expect("hyperlink run");
    assert_eq!(link.byte_range, (3, 7));
    assert!(
        link.link
            .as_ref()
            .is_some_and(|link| link.target.matches_display("res://docs/help"))
    );
    assert_eq!(link.style.underline, Some(true));
    assert!(link.style.color.is_some());
}

#[test]
fn text_rich_bbcode_image_and_url_share_inline_contracts() {
    let parsed = parse_rich_text(
        "[img=res://icons/star.png][url=res://docs/help]help[/url]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "\u{fffc}help");
    assert!(parsed.runs[0].inline.is_some());
    assert!(
        parsed.runs[1]
            .link
            .as_ref()
            .is_some_and(|link| link.target.matches_display("res://docs/help"))
    );
}

#[test]
fn text_rich_inline_resources_reject_network_and_escape_paths() {
    let parsed = parse_rich_text(
        "<img src=\"https://example.com/a.png\"><img src=\"res://../secret.png\"><a href=\"https://example.com\">plain</a>",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "plain");
    assert!(parsed.runs.iter().all(|run| run.inline.is_none()));
    assert!(parsed.runs.iter().all(|run| run.link.is_none()));
    assert!(
        parsed
            .runs
            .iter()
            .all(|run| run.style.underline != Some(true))
    );
}

#[test]
fn text_rich_bbcode_builtin_icon_emits_inline_metric_contract() {
    let parsed = parse_rich_text(
        "before[icon=res://icons/star.png|16x24|center|Favorite]after",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "before\u{fffc}after");
    assert!(matches!(
        parsed.runs[1].inline.as_ref(),
        Some(InlineObjectRef::Icon {
            size,
            baseline: InlineBaseline::Center,
            alternative_text: Some(alternative_text),
            ..
        }) if size.to_array() == [16.0, 24.0] && alternative_text == "Favorite"
    ));
}

#[test]
fn text_rich_bbcode_builtin_widget_emits_sized_placeholder_contract() {
    let parsed = parse_rich_text("a[widget=42|24x16]b", RichTextFormat::BbCodeV1);

    assert_eq!(parsed.text.as_ref(), "a\u{fffc}b");
    assert!(matches!(
        parsed.runs[1].inline.as_ref(),
        Some(InlineObjectRef::Widget { slot, size })
            if slot.value() == 42 && size.to_array() == [24.0, 16.0]
    ));
}
