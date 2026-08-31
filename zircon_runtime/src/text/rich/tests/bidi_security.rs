use std::sync::Arc;

use crate::text::{
    RichParseBudget, RichParseResult, RichTextAuthoringDiagnosticCode, RichTextContentTrust,
    RichTextFormat, RichTextParseError,
};

use super::{RichTextParser, parser_registry::parse_rich_text as try_parse_rich_text};

fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    try_parse_rich_text(markup, format).expect("test rich source fits parser budgets")
}

fn parse_trusted_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    RichParseResult::clone(
        RichTextParser::default()
            .compile_with_content_trust(markup, format, RichTextContentTrust::TrustedAuthoring)
            .expect("trusted test rich source fits parser budgets")
            .parsed(),
    )
}

#[test]
fn all_formats_report_source_ranged_bidi_controls_without_rewriting_text() {
    let plain_source = "a\u{200e}b\u{202a}c\u{202c}d\u{202e}e\u{202c}f\u{2066}g\u{2069}";
    let plain = parse_trusted_rich_text(plain_source, RichTextFormat::Plain);
    assert_eq!(plain.text.as_ref(), plain_source);
    assert_eq!(
        plain
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.source_range))
            .collect::<Vec<_>>(),
        vec![
            (RichTextAuthoringDiagnosticCode::BidirectionalMark, (1, 4)),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalEmbedding,
                (5, 8),
            ),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalEmbedding,
                (9, 12),
            ),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalOverride,
                (13, 16),
            ),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalEmbedding,
                (17, 20),
            ),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalIsolate,
                (21, 24),
            ),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalIsolate,
                (25, 28),
            ),
        ]
    );

    let markdown_source = "**a\u{202e}b\u{202c}**";
    let markdown = parse_trusted_rich_text(markdown_source, RichTextFormat::MarkdownInlineV1);
    assert_eq!(markdown.text.as_ref(), "a\u{202e}b\u{202c}");
    assert_eq!(
        markdown
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.source_range)
            .collect::<Vec<_>>(),
        vec![(3, 6), (7, 10)]
    );

    let html_source = "a&#x202e;b&#x202c;";
    let html = parse_trusted_rich_text(html_source, RichTextFormat::HtmlSubsetV1);
    assert_eq!(html.text.as_ref(), "a\u{202e}b\u{202c}");
    assert_eq!(
        html.authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.source_range)
            .collect::<Vec<_>>(),
        vec![(1, 9), (10, 18)]
    );

    let mixed_html_source = "&#x2066;a\u{2069}";
    let mixed_html = parse_rich_text(mixed_html_source, RichTextFormat::HtmlSubsetV1);
    assert_eq!(
        mixed_html
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.source_range)
            .collect::<Vec<_>>(),
        vec![(0, 8), (9, 12)]
    );

    let bbcode_source = "[rlo]x[pdf][lri]y[pdi]";
    let bbcode = parse_trusted_rich_text(bbcode_source, RichTextFormat::BbCodeV1);
    assert_eq!(bbcode.text.as_ref(), "\u{202e}x\u{202c}\u{2066}y\u{2069}");
    assert_eq!(
        bbcode
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.source_range)
            .collect::<Vec<_>>(),
        vec![(0, 5), (6, 11), (11, 16), (17, 22)]
    );
}

#[test]
fn untrusted_rich_text_rejects_legacy_bidi_controls_with_source_ranges() {
    let parser = RichTextParser::default();
    for (source, format, code, source_range) in [
        (
            "a\u{202e}b",
            RichTextFormat::Plain,
            RichTextAuthoringDiagnosticCode::BidirectionalOverride,
            (1, 4),
        ),
        (
            "&#x202e;",
            RichTextFormat::HtmlSubsetV1,
            RichTextAuthoringDiagnosticCode::BidirectionalOverride,
            (0, 8),
        ),
        (
            "[rlo]",
            RichTextFormat::BbCodeV1,
            RichTextAuthoringDiagnosticCode::BidirectionalOverride,
            (0, 5),
        ),
    ] {
        assert_eq!(
            parser.compile(source, format),
            Err(RichTextParseError::BidiControlNotAllowed { code, source_range })
        );
    }
}

#[test]
fn bidi_isolates_are_balanced_and_depth_bounded_for_all_trust_levels() {
    let parser = RichTextParser::default();
    let source = "\u{2066}logical\u{2069}";
    let untrusted = parser
        .compile(source, RichTextFormat::Plain)
        .expect("balanced isolate is safe for untrusted content");
    let trusted = parser
        .compile_with_content_trust(
            source,
            RichTextFormat::Plain,
            RichTextContentTrust::TrustedAuthoring,
        )
        .expect("balanced isolate is also accepted for trusted authoring");

    assert_eq!(untrusted.content_trust(), RichTextContentTrust::Untrusted);
    assert_eq!(
        trusted.content_trust(),
        RichTextContentTrust::TrustedAuthoring
    );
    assert!(!Arc::ptr_eq(&untrusted, &trusted));
    assert_eq!(parser.compiled_cache_report().parse_count, 2);
    assert_eq!(
        parser.compile("\u{2069}", RichTextFormat::Plain),
        Err(RichTextParseError::UnbalancedBidiControl {
            code: RichTextAuthoringDiagnosticCode::BidirectionalIsolate,
            source_range: (0, 3),
        })
    );

    let bounded =
        RichTextParser::with_budget(RichParseBudget::default().with_max_bidi_control_depth(1));
    assert_eq!(
        bounded.compile("\u{2066}\u{2068}x\u{2069}\u{2069}", RichTextFormat::Plain),
        Err(RichTextParseError::BidiControlDepthExceeded {
            attempted_depth: 2,
            max_depth: 1,
            source_range: (3, 6),
        })
    );
}

#[test]
fn trusted_legacy_controls_must_still_balance() {
    let parser = RichTextParser::default();
    parser
        .compile_with_content_trust(
            "\u{202e}logical\u{202c}",
            RichTextFormat::Plain,
            RichTextContentTrust::TrustedAuthoring,
        )
        .expect("balanced trusted override compiles");
    assert_eq!(
        parser.compile_with_content_trust(
            "\u{202e}logical",
            RichTextFormat::Plain,
            RichTextContentTrust::TrustedAuthoring,
        ),
        Err(RichTextParseError::UnbalancedBidiControl {
            code: RichTextAuthoringDiagnosticCode::BidirectionalOverride,
            source_range: (0, 3),
        })
    );
}

#[test]
fn bidi_control_diagnostics_share_the_authoring_budget() {
    let parser =
        RichTextParser::with_budget(RichParseBudget::default().with_max_authoring_diagnostics(2));
    let parsed = parser
        .parse("\u{200e}\u{200f}\u{061c}", RichTextFormat::Plain)
        .expect("directional marks fit untrusted parser policy");

    assert_eq!(parsed.authoring_diagnostics.len(), 2);
    assert!(parsed.authoring_diagnostics_truncated);
}
