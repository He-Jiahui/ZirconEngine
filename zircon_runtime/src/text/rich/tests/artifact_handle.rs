use super::*;
use crate::text::{RichTextFormat, RichTextParser};

#[test]
fn compiled_rich_artifact_identity_tracks_source_format_and_parser_generation() {
    let parser = RichTextParser::default();
    let first = parser
        .compile("[url=first]same[/url]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");
    let same = parser
        .compile("[url=first]same[/url]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");
    let different_source = parser
        .compile("[url=second]same[/url]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");
    let different_format = parser
        .compile("[url=first]same[/url]", RichTextFormat::Plain)
        .expect("test rich source fits parser budgets");
    let different_generation = RichTextParser::default()
        .compile("[url=first]same[/url]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");
    let first = register_compiled_rich_text_artifact(first);
    let same = register_compiled_rich_text_artifact(same);
    let different_source = register_compiled_rich_text_artifact(different_source);
    let different_format = register_compiled_rich_text_artifact(different_format);
    let different_generation = register_compiled_rich_text_artifact(different_generation);

    assert_eq!(first, same);
    assert_ne!(first, different_source);
    assert_ne!(first, different_format);
    assert_ne!(first, different_generation);
}
