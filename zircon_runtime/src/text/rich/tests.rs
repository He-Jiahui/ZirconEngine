use crate::core::{math::Vec2, resource::ResourceId};
use crate::text::TextAlign;
use crate::text::{
    FontFamilyName, InlineBaseline, InlineObjectRef, RichIconAssetId, RichParseBudget,
    RichParseResult, RichTextAuthoringDiagnosticCode, RichTextAuthoringDiagnosticSeverity,
    RichTextAuthoringRecovery, RichTextFormat, RichTextParseError, StyleOverride,
};

use super::{
    RichTextDecoration, RichTextDecorator, RichTextDecoratorRegistrationError, RichTextParser,
    parser_registry::{
        compile_rich_text, lookup_compiled_rich_text, parse_rich_text as try_parse_rich_text,
        shared_builtin_parser,
    },
};

fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    try_parse_rich_text(markup, format).expect("test rich source fits parser budgets")
}

fn parse_with_parser(
    parser: &RichTextParser,
    markup: &str,
    format: RichTextFormat,
) -> RichParseResult {
    parser
        .parse(markup, format)
        .expect("test rich source fits parser budgets")
}

struct AccentDecorator;

impl RichTextDecorator for AccentDecorator {
    fn tag(&self) -> &str {
        "accent"
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        if value != Some("strong") {
            return false;
        }
        decoration.style.weight = Some(800);
        decoration.style.underline = Some(true);
        true
    }
}

struct BadgeDecorator;

impl RichTextDecorator for BadgeDecorator {
    fn tag(&self) -> &str {
        "badge"
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        let Some(alternative_text) = value else {
            return false;
        };
        decoration.inline = Some(InlineObjectRef::Icon {
            asset: RichIconAssetId::from_resource_id(ResourceId::from_stable_label(
                "res://icons/custom-badge.png",
            )),
            size: Vec2::new(16.0, 16.0),
            baseline: InlineBaseline::Baseline,
            alternative_text: Some(alternative_text.to_owned()),
        });
        true
    }
}

#[test]
fn text_rich_bbcode_nested_styles_flatten_to_runs() {
    let parsed = parse_rich_text("[b]a[i]b[/i][/b]", RichTextFormat::BbCodeV1);

    assert_eq!(parsed.text.as_ref(), "ab");
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
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "red plain");
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
    let parsed = parse_rich_text("a[b]\u{0301}[/b]x", RichTextFormat::BbCodeV1);

    assert_eq!(parsed.text.as_ref(), "a\u{0301}x");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].byte_range, (0, 4));
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
}

#[test]
fn text_rich_markdown_inline_v1_contract() {
    let parsed = parse_rich_text(
        "plain **bold** *italic* `code`",
        RichTextFormat::MarkdownInlineV1,
    );

    assert_eq!(parsed.text.as_ref(), "plain bold italic code");
    assert_eq!(parsed.runs.len(), 6);
    assert_eq!(parsed.runs[1].style.weight, Some(700));
    assert_eq!(parsed.runs[3].style.italic, Some(true));
    assert_eq!(parsed.runs[5].style.code, Some(true));
}

#[test]
fn text_rich_html_and_bbcode_links_preserve_shared_tooltips() {
    let html = parse_rich_text(
        "<a href=\"res://docs/help.md\" title=\"Open help\">Help</a>",
        RichTextFormat::HtmlSubsetV1,
    );
    let bbcode = parse_rich_text(
        "[url href='res://docs/help.md' title='Open help']Help[/url]",
        RichTextFormat::BbCodeV1,
    );

    for parsed in [&html, &bbcode] {
        let link = parsed.runs[0]
            .link
            .as_ref()
            .expect("compiled link metadata");
        assert!(link.target.matches_display("res://docs/help.md"));
        assert_eq!(link.tooltip.as_deref(), Some("Open help"));
    }
}

mod parser_performance;

#[test]
fn text_rich_html_whitelist_drops_unknown_tags() {
    let parsed = parse_rich_text(
        "<script onclick=\"run()\">keep</script><b data-x=\"ignored\">bold</b>",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "keepbold");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
    assert_eq!(parsed.runs[1].style.weight, Some(700));
}

#[test]
fn text_rich_html_subset_reports_deterministic_structural_recovery() {
    let source = "<script>x</script></b><b><i>y</b>";
    let parsed = parse_rich_text(source, RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), "xy");
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            RichTextAuthoringDiagnosticCode::UnsupportedTag,
            RichTextAuthoringDiagnosticCode::UnsupportedTag,
            RichTextAuthoringDiagnosticCode::UnmatchedClosingTag,
            RichTextAuthoringDiagnosticCode::ImplicitlyClosedTag,
        ]
    );
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.source_range)
            .collect::<Vec<_>>(),
        vec![(0, 8), (9, 18), (18, 22), (29, 33)]
    );
    assert!(
        parsed.authoring_diagnostics.iter().all(|diagnostic| {
            diagnostic.severity == RichTextAuthoringDiagnosticSeverity::Warning
        })
    );
    assert_eq!(
        parsed.authoring_diagnostics[0].recovery,
        RichTextAuthoringRecovery::DroppedMarkup
    );
    assert_eq!(
        parsed.authoring_diagnostics[3].recovery,
        RichTextAuthoringRecovery::ImplicitlyClosed
    );
    assert!(!parsed.authoring_diagnostics_truncated);
}

#[test]
fn text_rich_html_subset_bounds_diagnostics_and_publishes_truncation_receipt() {
    let parser =
        RichTextParser::with_budget(RichParseBudget::default().with_max_authoring_diagnostics(2));
    let parsed = parse_with_parser(
        &parser,
        "<one>x</one><two>y</two><three>z</three>",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "xyz");
    assert_eq!(parsed.authoring_diagnostics.len(), 2);
    assert!(parsed.authoring_diagnostics_truncated);
}

#[test]
fn text_rich_html_subset_reports_unclosed_opening_source() {
    let parsed = parse_rich_text("<b>open", RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), "open");
    assert_eq!(parsed.authoring_diagnostics.len(), 1);
    let diagnostic = parsed.authoring_diagnostics[0];
    assert_eq!(
        diagnostic.code,
        RichTextAuthoringDiagnosticCode::UnclosedTag
    );
    assert_eq!(diagnostic.source_range, (0, 3));
    assert_eq!(
        diagnostic.recovery,
        RichTextAuthoringRecovery::ClosedAtEndOfInput
    );
}

#[test]
fn text_rich_authoring_diagnostic_codes_and_catalog_keys_are_stable_and_unique() {
    let codes = [
        RichTextAuthoringDiagnosticCode::UnsupportedTag,
        RichTextAuthoringDiagnosticCode::UnmatchedClosingTag,
        RichTextAuthoringDiagnosticCode::ImplicitlyClosedTag,
        RichTextAuthoringDiagnosticCode::UnclosedTag,
        RichTextAuthoringDiagnosticCode::UnsupportedAttribute,
        RichTextAuthoringDiagnosticCode::MalformedAttribute,
        RichTextAuthoringDiagnosticCode::InvalidAttributeValue,
        RichTextAuthoringDiagnosticCode::UnsupportedStyleProperty,
        RichTextAuthoringDiagnosticCode::MalformedTag,
        RichTextAuthoringDiagnosticCode::UnterminatedQuotedAttribute,
        RichTextAuthoringDiagnosticCode::MalformedEntity,
        RichTextAuthoringDiagnosticCode::UnrecognizedEntity,
        RichTextAuthoringDiagnosticCode::BidirectionalMark,
        RichTextAuthoringDiagnosticCode::BidirectionalEmbedding,
        RichTextAuthoringDiagnosticCode::BidirectionalOverride,
        RichTextAuthoringDiagnosticCode::BidirectionalIsolate,
    ];
    let diagnostic_codes = codes.map(RichTextAuthoringDiagnosticCode::diagnostic_code);
    let message_keys = codes.map(RichTextAuthoringDiagnosticCode::message_key);

    for (index, diagnostic_code) in diagnostic_codes.iter().enumerate() {
        assert!(diagnostic_code.starts_with("ZR-TEXT-RICH-AUTHOR-"));
        assert!(!diagnostic_codes[..index].contains(diagnostic_code));
    }
    for (index, message_key) in message_keys.iter().enumerate() {
        assert!(message_key.starts_with("text.rich.author."));
        assert!(!message_keys[..index].contains(message_key));
    }
}

#[test]
fn text_rich_html_subset_reports_attribute_and_style_recovery_in_one_parse() {
    let source = concat!(
        "<b onclick=\"run\">a</b>",
        "<font size=\"-2\">b</font>",
        "<span style=\"position:absolute;color:nope\">c</span>",
        "<img src=\"https://example.com/a.png\">"
    );
    let parsed = parse_rich_text(source, RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), "abc");
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            RichTextAuthoringDiagnosticCode::UnsupportedAttribute,
            RichTextAuthoringDiagnosticCode::InvalidAttributeValue,
            RichTextAuthoringDiagnosticCode::UnsupportedStyleProperty,
            RichTextAuthoringDiagnosticCode::InvalidAttributeValue,
            RichTextAuthoringDiagnosticCode::InvalidAttributeValue,
        ]
    );
    assert!(parsed.authoring_diagnostics.iter().all(|diagnostic| {
        let range = diagnostic.source_range.0 as usize..diagnostic.source_range.1 as usize;
        source
            .get(range)
            .is_some_and(|token| token.starts_with('<'))
    }));
}

#[test]
fn text_rich_html_subset_reports_malformed_attribute_recovery() {
    let parsed = parse_rich_text("<b =x>value</b>", RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), "value");
    assert!(parsed.authoring_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == RichTextAuthoringDiagnosticCode::MalformedAttribute
            && diagnostic.recovery == RichTextAuthoringRecovery::IgnoredAttribute
    }));
}

#[test]
fn text_rich_html_subset_preserves_malformed_markup_and_entity_source() {
    let source = concat!(
        "before<b@>x",
        "<b title=\"unterminated>after ",
        "&bogus; &#xZZ; &tail"
    );
    let parsed = parse_rich_text(source, RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), source);
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            RichTextAuthoringDiagnosticCode::MalformedTag,
            RichTextAuthoringDiagnosticCode::UnterminatedQuotedAttribute,
            RichTextAuthoringDiagnosticCode::UnrecognizedEntity,
            RichTextAuthoringDiagnosticCode::MalformedEntity,
        ]
    );
    assert!(parsed.authoring_diagnostics.iter().all(|diagnostic| {
        diagnostic.recovery == RichTextAuthoringRecovery::PreservedAsText
            && diagnostic.source_range.0 < diagnostic.source_range.1
    }));
}

#[test]
fn text_rich_html_subset_does_not_diagnose_plain_less_than_text() {
    let parsed = parse_rich_text("one < two", RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), "one < two");
    assert!(parsed.authoring_diagnostics.is_empty());
}

#[test]
fn text_rich_html_subset_orders_entity_before_eof_malformed_tag() {
    let source = "&bogus; <b";
    let parsed = parse_rich_text(source, RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), source);
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            RichTextAuthoringDiagnosticCode::UnrecognizedEntity,
            RichTextAuthoringDiagnosticCode::MalformedTag,
        ]
    );
}

#[test]
fn text_rich_html_subset_reports_eof_unterminated_attribute_quote() {
    let source = "before<b title=\"unterminated";
    let parsed = parse_rich_text(source, RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), source);
    assert_eq!(parsed.authoring_diagnostics.len(), 1);
    assert_eq!(
        parsed.authoring_diagnostics[0].code,
        RichTextAuthoringDiagnosticCode::UnterminatedQuotedAttribute
    );
    assert_eq!(
        parsed.authoring_diagnostics[0].recovery,
        RichTextAuthoringRecovery::PreservedAsText
    );
}

#[test]
fn text_rich_html_entities_decode() {
    let parsed = parse_rich_text(
        "A &amp; B &#x4E2D; &#25991; &lt;ok&gt;",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "A & B 中 文 <ok>");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].byte_range, (0, parsed.text.len() as u32));
}

#[test]
fn text_rich_html_br_forces_break() {
    let parsed = parse_rich_text("first<br>second<br/>third", RichTextFormat::HtmlSubsetV1);

    assert_eq!(parsed.text.as_ref(), "first\nsecond\nthird");
    assert_eq!(parsed.runs.len(), 1);
}

#[test]
fn text_rich_html_span_style_accepts_only_controlled_properties() {
    let parsed = parse_rich_text(
        "<span style=\"color:#0f0; font-size:18px; font-weight:650; font-style:italic; text-decoration:underline line-through; background:url(evil)\">safe</span>",
        RichTextFormat::HtmlSubsetV1,
    );

    assert_eq!(parsed.text.as_ref(), "safe");
    assert_eq!(parsed.runs.len(), 1);
    let style = &parsed.runs[0].style;
    assert_eq!(style.color.unwrap().to_array(), [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(style.font_size, Some(18.0));
    assert_eq!(style.weight, Some(650));
    assert_eq!(style.italic, Some(true));
    assert_eq!(style.underline, Some(true));
    assert_eq!(style.strike, Some(true));
    assert_eq!(style.bg_color, None);
}

#[test]
fn text_rich_bbcode_block_alignment_emits_paragraph_overrides() {
    let parsed = parse_rich_text(
        "[center]alpha\nbeta[/center][right]gamma[/right]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "alpha\nbeta\ngamma");
    assert_eq!(parsed.paragraphs.len(), 2);
    assert_eq!(parsed.paragraphs[0].0, (0, 10));
    assert_eq!(parsed.paragraphs[0].1.align, Some(TextAlign::Center));
    assert_eq!(parsed.paragraphs[1].0, (11, 16));
    assert_eq!(parsed.paragraphs[1].1.align, Some(TextAlign::Right));
}

#[test]
fn text_rich_bbcode_left_and_fill_emit_shared_paragraph_overrides() {
    let parsed = parse_rich_text(
        "[left]alpha[/left][fill]beta[/fill]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "alpha\nbeta");
    assert_eq!(parsed.paragraphs.len(), 2);
    assert_eq!(parsed.paragraphs[0].1.align, Some(TextAlign::Left));
    assert_eq!(parsed.paragraphs[1].1.align, Some(TextAlign::Justify));
}

#[test]
fn text_rich_bbcode_literal_and_bidi_control_tags_emit_unicode_text() {
    let parsed = parse_rich_text(
        "[lb]tag[rb][br][lri]עברית[pdi][shy]word",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(
        parsed.text.as_ref(),
        "[tag]\n\u{2066}עברית\u{2069}\u{00ad}word"
    );
    assert_eq!(
        parsed
            .authoring_diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.source_range))
            .collect::<Vec<_>>(),
        vec![
            (
                RichTextAuthoringDiagnosticCode::BidirectionalIsolate,
                (15, 20)
            ),
            (
                RichTextAuthoringDiagnosticCode::BidirectionalIsolate,
                (30, 35)
            ),
        ]
    );
    assert!(
        parsed.authoring_diagnostics.iter().all(|diagnostic| {
            diagnostic.recovery == RichTextAuthoringRecovery::PreservedAsText
        })
    );
}

#[test]
fn text_rich_custom_decorator_registration_applies_style_without_parser_branch() {
    let mut parser = RichTextParser::default();
    parser
        .register_decorator(AccentDecorator)
        .expect("custom decorator registration");

    let parsed = parse_with_parser(
        &parser,
        "plain [accent=strong]custom[/accent] tail",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "plain custom tail");
    let custom = parsed
        .runs
        .iter()
        .find(|run| run.byte_range == (6, 12))
        .expect("custom styled run");
    assert_eq!(custom.style.weight, Some(800));
    assert_eq!(custom.style.underline, Some(true));
}

#[test]
fn text_rich_custom_decorator_can_emit_inline_object_contract() {
    let mut parser = RichTextParser::default();
    parser
        .register_decorator(BadgeDecorator)
        .expect("badge decorator registration");

    let parsed = parse_with_parser(&parser, "a[badge=★]b", RichTextFormat::BbCodeV1);

    assert_eq!(parsed.text.as_ref(), "a\u{fffc}b");
    let inline = parsed
        .runs
        .iter()
        .find_map(|run| run.inline.as_ref())
        .expect("custom inline run");
    assert!(matches!(
        inline,
        InlineObjectRef::Icon {
            alternative_text: Some(alternative_text),
            ..
        } if alternative_text == "★"
    ));
}

#[test]
fn text_rich_custom_decorator_rejects_builtin_tag_shadowing() {
    struct BuiltinShadow;

    impl RichTextDecorator for BuiltinShadow {
        fn tag(&self) -> &str {
            "b"
        }

        fn decorate(&self, _value: Option<&str>, _decoration: &mut RichTextDecoration) -> bool {
            true
        }
    }

    let mut parser = RichTextParser::default();
    assert_eq!(
        parser.register_decorator(BuiltinShadow),
        Err(RichTextDecoratorRegistrationError::DuplicateTag(
            "b".to_string()
        ))
    );
}

#[test]
fn text_rich_custom_decorator_rejects_parser_reserved_tag_shadowing() {
    struct ReservedShadow;

    impl RichTextDecorator for ReservedShadow {
        fn tag(&self) -> &str {
            "img"
        }

        fn decorate(&self, _value: Option<&str>, _decoration: &mut RichTextDecoration) -> bool {
            true
        }
    }

    let mut parser = RichTextParser::default();
    assert_eq!(
        parser.register_decorator(ReservedShadow),
        Err(RichTextDecoratorRegistrationError::DuplicateTag(
            "img".to_string()
        ))
    );

    struct LiteralShadow;

    impl RichTextDecorator for LiteralShadow {
        fn tag(&self) -> &str {
            "br"
        }

        fn decorate(&self, _value: Option<&str>, _decoration: &mut RichTextDecoration) -> bool {
            true
        }
    }

    assert_eq!(
        parser.register_decorator(LiteralShadow),
        Err(RichTextDecoratorRegistrationError::DuplicateTag(
            "br".to_string()
        ))
    );
}

#[test]
fn text_rich_custom_decorator_rejects_invalid_tag_name() {
    struct InvalidTag;

    impl RichTextDecorator for InvalidTag {
        fn tag(&self) -> &str {
            "bad tag"
        }

        fn decorate(&self, _value: Option<&str>, _decoration: &mut RichTextDecoration) -> bool {
            true
        }
    }

    let mut parser = RichTextParser::default();
    assert_eq!(
        parser.register_decorator(InvalidTag),
        Err(RichTextDecoratorRegistrationError::InvalidTag(
            "bad tag".to_string()
        ))
    );
}

#[test]
fn text_rich_rejected_custom_decorator_preserves_inner_text_without_style() {
    let mut parser = RichTextParser::default();
    parser
        .register_decorator(AccentDecorator)
        .expect("custom decorator registration");

    let parsed = parse_with_parser(
        &parser,
        "before [accent=weak]safe[/accent] after",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "before safe after");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
}

#[test]
fn text_rich_bbcode_known_emoji_shortcode_expands_inside_active_style() {
    let parsed = parse_rich_text("[b]go :rocket:[/b]", RichTextFormat::BbCodeV1);

    assert_eq!(parsed.text.as_ref(), "go 🚀");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].style.weight, Some(700));
}

#[test]
fn text_rich_bbcode_unknown_emoji_shortcode_is_preserved() {
    let parsed = parse_rich_text(
        "keep :zircon_unknown: then :rocket:",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "keep :zircon_unknown: then 🚀");
}

#[test]
fn text_rich_custom_emoji_shortcode_registration_is_parser_local() {
    let mut parser = RichTextParser::default();
    parser
        .register_emoji_shortcode("zircon", "💎")
        .expect("custom shortcode registration");

    assert_eq!(
        parse_with_parser(&parser, ":zircon:", RichTextFormat::BbCodeV1)
            .text
            .as_ref(),
        "💎"
    );
    assert_eq!(
        parse_with_parser(
            &RichTextParser::default(),
            ":zircon:",
            RichTextFormat::BbCodeV1,
        )
        .text
        .as_ref(),
        ":zircon:"
    );
}

#[test]
fn text_rich_custom_emoji_shortcode_rejects_builtin_shadowing() {
    let mut parser = RichTextParser::default();

    assert!(parser.register_emoji_shortcode("rocket", "🛸").is_err());
}

#[test]
fn text_rich_convenience_parser_reuses_builtin_registries() {
    assert!(std::ptr::eq(
        shared_builtin_parser(),
        shared_builtin_parser()
    ));
}

#[test]
fn text_rich_consumers_can_only_lookup_an_existing_compiled_artifact() {
    let markup = "[b]shared consumer artifact[/b]";
    let compiled = compile_rich_text(markup, RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");
    let looked_up = lookup_compiled_rich_text(markup, RichTextFormat::BbCodeV1)
        .expect("the UI-owned compiled artifact should be available");

    assert!(std::sync::Arc::ptr_eq(&compiled, &looked_up));
    assert!(
        lookup_compiled_rich_text(
            "[b]consumer lookup cannot compile this missing document[/b]",
            RichTextFormat::BbCodeV1,
        )
        .is_none()
    );
}

#[test]
fn text_rich_grapheme_alignment_uses_a_monotonic_run_cursor() {
    let source = include_str!("parser/run_alignment.rs");
    let start = source
        .find("fn align_runs_to_graphemes_bounded")
        .expect("alignment function");
    let end = source[start..]
        .find("\nfn ascii_runs_are_canonical")
        .map(|offset| start + offset)
        .expect("alignment function end");
    let body = &source[start..end];

    assert!(body.contains("let mut run_index"));
    assert!(body.contains("while run_index"));
    assert!(!body.contains(".iter()\n            .find"));
}

#[test]
fn text_rich_plain_segments_borrow_when_no_replacement_is_needed() {
    assert!(matches!(
        super::html_subset::decode_entities("plain text"),
        std::borrow::Cow::Borrowed("plain text")
    ));
    let emoji = super::emoji_shortcode::EmojiShortcodeRegistry::with_builtins();
    assert!(matches!(
        emoji
            .expand("plain text", 0, usize::MAX)
            .expect("plain text fits output budget"),
        std::borrow::Cow::Borrowed("plain text")
    ));
}
