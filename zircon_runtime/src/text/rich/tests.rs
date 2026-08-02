use crate::text::TextAlign;
use crate::text::{FontFamilyName, InlineBaseline, InlineObjectRef, RichTextFormat, StyleOverride};

use super::{
    compile_rich_text, lookup_compiled_rich_text, parse_rich_text, shared_builtin_parser,
    RichTextDecoration, RichTextDecorator, RichTextDecoratorRegistrationError, RichTextParser,
};

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
        let Some(glyph) = value.and_then(|value| value.chars().next()) else {
            return false;
        };
        decoration.inline = Some(InlineObjectRef::Icon {
            glyph,
            font: FontFamilyName::from("Zircon Icons"),
        });
        true
    }
}

#[test]
fn text_rich_bbcode_nested_styles_flatten_to_runs() {
    let parsed = parse_rich_text("[b]a[i]b[/i][/b]", RichTextFormat::BbCode);

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
        RichTextFormat::BbCode,
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
    let parsed = parse_rich_text("a[b]\u{0301}[/b]x", RichTextFormat::BbCode);

    assert_eq!(parsed.text.as_ref(), "a\u{0301}x");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].byte_range, (0, 4));
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
}

#[test]
fn text_rich_markdown_compat_unchanged() {
    let parsed = parse_rich_text("plain **bold** *italic* `code`", RichTextFormat::Markdown);

    assert_eq!(parsed.text.as_ref(), "plain bold italic code");
    assert_eq!(parsed.runs.len(), 6);
    assert_eq!(parsed.runs[1].style.weight, Some(700));
    assert_eq!(parsed.runs[3].style.italic, Some(true));
    assert_eq!(parsed.runs[5].style.code, Some(true));
}

#[test]
fn text_rich_html_whitelist_drops_unknown_tags() {
    let parsed = parse_rich_text(
        "<script onclick=\"run()\">keep</script><b data-x=\"ignored\">bold</b>",
        RichTextFormat::Html,
    );

    assert_eq!(parsed.text.as_ref(), "keepbold");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
    assert_eq!(parsed.runs[1].style.weight, Some(700));
}

#[test]
fn text_rich_html_entities_decode() {
    let parsed = parse_rich_text(
        "A &amp; B &#x4E2D; &#25991; &lt;ok&gt;",
        RichTextFormat::Html,
    );

    assert_eq!(parsed.text.as_ref(), "A & B 中 文 <ok>");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].byte_range, (0, parsed.text.len() as u32));
}

#[test]
fn text_rich_html_br_forces_break() {
    let parsed = parse_rich_text("first<br>second<br/>third", RichTextFormat::Html);

    assert_eq!(parsed.text.as_ref(), "first\nsecond\nthird");
    assert_eq!(parsed.runs.len(), 1);
}

#[test]
fn text_rich_html_span_style_accepts_only_controlled_properties() {
    let parsed = parse_rich_text(
        "<span style=\"color:#0f0; font-size:18px; font-weight:650; font-style:italic; text-decoration:underline line-through; background:url(evil)\">safe</span>",
        RichTextFormat::Html,
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
fn text_rich_inline_image_parses_placeholder_metric_contract() {
    let parsed = parse_rich_text(
        "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\" baseline=\"baseline\">after",
        RichTextFormat::Html,
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
            ..
        }) if size.to_array() == [16.0, 24.0]
    ));
}

#[test]
fn text_rich_hyperlink_carries_href_and_hit_range() {
    let parsed = parse_rich_text(
        "go <a href=\"res://docs/help\">help</a> now",
        RichTextFormat::Html,
    );

    assert_eq!(parsed.text.as_ref(), "go help now");
    let link = parsed
        .runs
        .iter()
        .find(|run| run.link.is_some())
        .expect("hyperlink run");
    assert_eq!(link.byte_range, (3, 7));
    assert_eq!(
        link.link.as_ref().map(|link| link.href.as_str()),
        Some("res://docs/help")
    );
    assert_eq!(link.style.underline, Some(true));
    assert!(link.style.color.is_some());
}

#[test]
fn text_rich_bbcode_image_and_url_share_inline_contracts() {
    let parsed = parse_rich_text(
        "[img=res://icons/star.png][url=res://docs/help]help[/url]",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text.as_ref(), "\u{fffc}help");
    assert!(parsed.runs[0].inline.is_some());
    assert_eq!(
        parsed.runs[1].link.as_ref().map(|link| link.href.as_str()),
        Some("res://docs/help")
    );
}

#[test]
fn text_rich_inline_resources_reject_network_and_escape_paths() {
    let parsed = parse_rich_text(
        "<img src=\"https://example.com/a.png\"><img src=\"res://../secret.png\"><a href=\"https://example.com\">plain</a>",
        RichTextFormat::Html,
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
fn text_rich_bbcode_block_alignment_emits_paragraph_overrides() {
    let parsed = parse_rich_text(
        "[center]alpha\nbeta[/center][right]gamma[/right]",
        RichTextFormat::BbCode,
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
        RichTextFormat::BbCode,
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
        RichTextFormat::BbCode,
    );

    assert_eq!(
        parsed.text.as_ref(),
        "[tag]\n\u{2066}עברית\u{2069}\u{00ad}word"
    );
}

#[test]
fn text_rich_custom_decorator_registration_applies_style_without_parser_branch() {
    let mut parser = RichTextParser::default();
    parser
        .register_decorator(AccentDecorator)
        .expect("custom decorator registration");

    let parsed = parser.parse(
        "plain [accent=strong]custom[/accent] tail",
        RichTextFormat::BbCode,
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

    let parsed = parser.parse("a[badge=★]b", RichTextFormat::BbCode);

    assert_eq!(parsed.text.as_ref(), "a\u{fffc}b");
    let inline = parsed
        .runs
        .iter()
        .find_map(|run| run.inline.as_ref())
        .expect("custom inline run");
    assert!(matches!(
        inline,
        InlineObjectRef::Icon { glyph: '★', font }
            if font.as_str() == "Zircon Icons"
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

    let parsed = parser.parse(
        "before [accent=weak]safe[/accent] after",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text.as_ref(), "before safe after");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].style, StyleOverride::default());
}

#[test]
fn text_rich_bbcode_builtin_icon_emits_inline_metric_contract() {
    let parsed = parse_rich_text("before[icon=★|Zircon Icons]after", RichTextFormat::BbCode);

    assert_eq!(parsed.text.as_ref(), "before\u{fffc}after");
    assert!(matches!(
        parsed.runs[1].inline.as_ref(),
        Some(InlineObjectRef::Icon { glyph: '★', font })
            if font.as_str() == "Zircon Icons"
    ));
}

#[test]
fn text_rich_bbcode_builtin_widget_emits_sized_placeholder_contract() {
    let parsed = parse_rich_text("a[widget=42|24x16]b", RichTextFormat::BbCode);

    assert_eq!(parsed.text.as_ref(), "a\u{fffc}b");
    assert!(matches!(
        parsed.runs[1].inline.as_ref(),
        Some(InlineObjectRef::Widget { id: 42, size })
            if size.to_array() == [24.0, 16.0]
    ));
}

#[test]
fn text_rich_bbcode_known_emoji_shortcode_expands_inside_active_style() {
    let parsed = parse_rich_text("[b]go :rocket:[/b]", RichTextFormat::BbCode);

    assert_eq!(parsed.text.as_ref(), "go 🚀");
    assert_eq!(parsed.runs.len(), 1);
    assert_eq!(parsed.runs[0].style.weight, Some(700));
}

#[test]
fn text_rich_bbcode_unknown_emoji_shortcode_is_preserved() {
    let parsed = parse_rich_text(
        "keep :zircon_unknown: then :rocket:",
        RichTextFormat::BbCode,
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
        parser
            .parse(":zircon:", RichTextFormat::BbCode)
            .text
            .as_ref(),
        "💎"
    );
    assert_eq!(
        RichTextParser::default()
            .parse(":zircon:", RichTextFormat::BbCode)
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
    let compiled = compile_rich_text(markup, RichTextFormat::BbCode);
    let looked_up = lookup_compiled_rich_text(markup, RichTextFormat::BbCode)
        .expect("the UI-owned compiled artifact should be available");

    assert!(std::sync::Arc::ptr_eq(&compiled, &looked_up));
    assert!(lookup_compiled_rich_text(
        "[b]consumer lookup cannot compile this missing document[/b]",
        RichTextFormat::BbCode,
    )
    .is_none());
}

#[test]
fn text_rich_grapheme_alignment_uses_a_monotonic_run_cursor() {
    let source = include_str!("parser.rs");
    let start = source
        .find("fn align_runs_to_graphemes")
        .expect("alignment function");
    let end = source[start..]
        .find("\nfn range_contains")
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
        emoji.expand("plain text"),
        std::borrow::Cow::Borrowed("plain text")
    ));
}
