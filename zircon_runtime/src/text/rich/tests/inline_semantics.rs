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
    assert!(link
        .link
        .as_ref()
        .is_some_and(|link| link.target.matches_display("res://docs/help")));
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
    assert!(parsed.runs[1]
        .link
        .as_ref()
        .is_some_and(|link| link.target.matches_display("res://docs/help")));
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
    assert!(parsed
        .runs
        .iter()
        .all(|run| run.style.underline != Some(true)));
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

#[test]
fn markdown_escaped_punctuation_keeps_literal_content_and_delimiter_boundaries() {
    for (source, expected) in [
        (r"\*literal\*", "*literal*"),
        (r"\*\*literal\*\*", "**literal**"),
        (r"\_\[\]\(\)", "_[]()"),
        (r"\\", "\\"),
        (r"\q tail\", "\\q tail\\"),
    ] {
        let parsed = parse_rich_text(source, RichTextFormat::MarkdownInlineV1);
        assert_eq!(parsed.text.as_ref(), expected);
        assert!(parsed
            .runs
            .iter()
            .all(|run| run.style == Default::default()));
    }
    for (source, expected, kind) in [
        (r"**left \** right**", "left ** right", 0),
        (r"*left \* right*", "left * right", 1),
        ("\x60left \\\x60 right\x60", "left \x60 right", 2),
    ] {
        let parsed = parse_rich_text(source, RichTextFormat::MarkdownInlineV1);
        assert_eq!(parsed.text.as_ref(), expected);
        assert_eq!(parsed.runs.len(), 1);
        let style = &parsed.runs[0].style;
        match kind {
            0 => assert_eq!(style.weight, Some(700)),
            1 => assert_eq!(style.italic, Some(true)),
            _ => assert_eq!(style.code, Some(true)),
        }
    }
    let parsed = parse_rich_text(r"\\*italic*", RichTextFormat::MarkdownInlineV1);
    assert_eq!(parsed.text.as_ref(), "\\italic");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].style.italic, None);
    assert_eq!(parsed.runs[1].style.italic, Some(true));
    // Escape syntax belongs only to MarkdownInlineV1.
    for format in [
        RichTextFormat::Plain,
        RichTextFormat::HtmlSubsetV1,
        RichTextFormat::BbCodeV1,
    ] {
        assert_eq!(
            parse_rich_text(r"\*literal\*", format).text.as_ref(),
            r"\*literal\*"
        );
    }
}

#[test]
fn markdown_escaped_prefixes_preserve_source_bidi_ranges_and_graphemes() {
    use crate::text::{
        RichTextAuthoringDiagnosticCode, RichTextContentTrust, RichTextParseError, RichTextParser,
    };
    use unicode_segmentation::UnicodeSegmentation;

    let source = "**\\*e\u{0301}\u{202e}x\u{202c}**";
    let parser = RichTextParser::default();
    let compiled = parser
        .compile_with_content_trust(
            source,
            RichTextFormat::MarkdownInlineV1,
            RichTextContentTrust::TrustedAuthoring,
        )
        .unwrap();
    let parsed = compiled.parsed();
    assert_eq!(parsed.text.as_ref(), "*e\u{0301}\u{202e}x\u{202c}");
    let override_start = source.find('\u{202e}').unwrap() as u32;
    let pop_start = source.find('\u{202c}').unwrap() as u32;
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.source_range)
            .collect::<Vec<_>>(),
        vec![
            (override_start, override_start + 3),
            (pop_start, pop_start + 3)
        ],
    );
    assert_eq!(
        parser.compile(source, RichTextFormat::MarkdownInlineV1),
        Err(RichTextParseError::BidiControlNotAllowed {
            code: RichTextAuthoringDiagnosticCode::BidirectionalOverride,
            source_range: (override_start, override_start + 3),
        },)
    );
    let parsed = parse_rich_text("e**\\*\u{0301}**", RichTextFormat::MarkdownInlineV1);
    assert_eq!(parsed.text.as_ref(), "e*\u{0301}");
    let boundaries = parsed
        .text
        .grapheme_indices(true)
        .map(|(index, _)| index as u32)
        .chain(std::iter::once(parsed.text.len() as u32))
        .collect::<Vec<_>>();
    for run in &parsed.runs {
        assert!(boundaries.contains(&run.byte_range.0));
        assert!(boundaries.contains(&run.byte_range.1));
    }
    assert!(parsed
        .runs
        .iter()
        .any(|run| run.byte_range == (1, 4) && run.style.weight == Some(700)));
}

#[test]
fn markdown_escape_tokens_and_decoded_output_respect_parse_budgets() {
    use crate::text::{RichParseBudget, RichTextParseError, RichTextParser};
    let format = RichTextFormat::MarkdownInlineV1;
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_tokens: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        parser.compile(r"\*\*", format),
        Err(RichTextParseError::TokenBudgetExceeded {
            attempted_tokens: 2,
            max_tokens: 1,
        })
    );
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_token_bytes: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        parser.compile(r"\*", format),
        Err(RichTextParseError::TokenByteBudgetExceeded {
            attempted_bytes: 2,
            max_bytes: 1,
        })
    );
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_output_bytes: 0,
        ..RichParseBudget::default()
    });
    assert_eq!(
        parser.compile(r"\*", format),
        Err(RichTextParseError::OutputByteBudgetExceeded {
            attempted_bytes: 1,
            max_bytes: 0,
        })
    );
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        parser.compile(r"\*", format),
        Err(RichTextParseError::SourceByteBudgetExceeded {
            actual_bytes: 2,
            max_bytes: 1,
        })
    );
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_runs: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        parser.compile(r"\* **x**", format),
        Err(RichTextParseError::RunCountBudgetExceeded {
            attempted_runs: 2,
            max_runs: 1,
        })
    );
}

#[test]
fn markdown_escaped_only_closers_finish_at_scale_without_formatting() {
    use crate::text::{RichParseBudget, RichTextParser};
    const ESCAPED_CLOSERS: usize = 100_000;
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_tokens: ESCAPED_CLOSERS * 2 + 2,
        ..RichParseBudget::default()
    });
    for (open, escaped, literal) in [
        ("*", r"\* ", "* "),
        ("**", r"\*\* ", "** "),
        ("\x60", "\\\x60 ", "\x60 "),
    ] {
        let source = format!("{open}{}", escaped.repeat(ESCAPED_CLOSERS));
        let compiled = parser
            .compile(&source, RichTextFormat::MarkdownInlineV1)
            .unwrap();
        let parsed = compiled.parsed();
        assert_eq!(
            parsed.text.as_ref(),
            format!("{open}{}", literal.repeat(ESCAPED_CLOSERS))
        );
        assert_eq!(parsed.runs.len(), 1);
        assert_eq!(parsed.runs[0].style, Default::default());
    }
}
