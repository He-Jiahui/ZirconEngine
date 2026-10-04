use super::*;
use crate::text::{register_compiled_rich_text_artifact, RichTextParser};

#[test]
fn semantic_projection_retains_compiled_visible_text_and_generation() {
    let parser = RichTextParser::default();
    let compiled = parser
        .compile("<b>Visible</b> text", RichTextFormat::HtmlSubsetV1)
        .expect("test rich source fits parser budgets");
    let handle = register_compiled_rich_text_artifact(compiled);

    let first = resolve_rich_semantic_projection(
        &handle,
        "<b>Visible</b> text",
        RichTextFormat::HtmlSubsetV1,
    )
    .expect("matching artifact projects semantics");
    let repeated = resolve_rich_semantic_projection(
        &handle,
        "<b>Visible</b> text",
        RichTextFormat::HtmlSubsetV1,
    )
    .expect("matching artifact projects semantics");

    assert_eq!(first.visible_text(), "Visible text");
    assert!(first.shares_source_generation(&repeated));
}

#[test]
fn semantic_projection_replaces_inline_image_placeholder_with_compiled_fallback() {
    let parser = RichTextParser::default();
    let compiled = parser
        .compile(
            "Before <img src=\"res://icons/star.png\" alt=\"favorite\"> after",
            RichTextFormat::HtmlSubsetV1,
        )
        .expect("test rich source fits parser budgets");
    let projection = from_compiled_rich_semantic_projection(
        compiled,
        "Before <img src=\"res://icons/star.png\" alt=\"favorite\"> after",
        RichTextFormat::HtmlSubsetV1,
    )
    .expect("matching artifact projects semantics");

    assert_eq!(projection.visible_text(), "Before favorite after");
}

#[test]
fn semantic_projection_respects_decorative_empty_alt_before_tooltip() {
    let parser = RichTextParser::default();
    let compiled = parser
        .compile(
            "Before <img src=\"res://icons/star.png\" alt=\"\" title=\"not a fallback\"> after",
            RichTextFormat::HtmlSubsetV1,
        )
        .expect("test rich source fits parser budgets");
    let projection = from_compiled_rich_semantic_projection(
        compiled,
        "Before <img src=\"res://icons/star.png\" alt=\"\" title=\"not a fallback\"> after",
        RichTextFormat::HtmlSubsetV1,
    )
    .expect("matching artifact projects semantics");

    assert_eq!(projection.visible_text(), "Before  after");
}

#[test]
fn semantic_projection_repeats_fallback_for_merged_adjacent_inline_runs() {
    let parser = RichTextParser::default();
    let source =
        "<img src=\"res://icons/star.png\" alt=\"A\"><img src=\"res://icons/star.png\" alt=\"A\">";
    let compiled = parser
        .compile(source, RichTextFormat::HtmlSubsetV1)
        .expect("test rich source fits parser budgets");
    let projection =
        from_compiled_rich_semantic_projection(compiled, source, RichTextFormat::HtmlSubsetV1)
            .expect("matching artifact projects semantics");

    assert_eq!(projection.visible_text(), "AA");
}

#[test]
fn semantic_projection_rejects_stale_source_and_format() {
    let compiled = RichTextParser::default()
        .compile("[b]Visible[/b]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");
    let handle = register_compiled_rich_text_artifact(compiled);

    assert!(
        resolve_rich_semantic_projection(&handle, "[b]Changed[/b]", RichTextFormat::BbCodeV1,)
            .is_none()
    );
    assert!(resolve_rich_semantic_projection(
        &handle,
        "[b]Visible[/b]",
        RichTextFormat::HtmlSubsetV1,
    )
    .is_none());
}
