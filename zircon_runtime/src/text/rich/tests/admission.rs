use crate::text::{
    LinkRef, RichParseBudget, RichTextDecoration, RichTextDecorator, RichTextFormat,
    RichTextParseError, RichTextParser,
};
use zircon_runtime_interface::ui::text::UiRichLinkTarget;

struct PanickingDecorator;

impl RichTextDecorator for PanickingDecorator {
    fn tag(&self) -> &str {
        "panic"
    }

    fn decorate(&self, _value: Option<&str>, _decoration: &mut RichTextDecoration) -> bool {
        panic!("decorator failure")
    }
}

struct OversizedMetadataDecorator;

impl RichTextDecorator for OversizedMetadataDecorator {
    fn tag(&self) -> &str {
        "oversized"
    }

    fn decorate(&self, _value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        decoration.link = Some(LinkRef {
            target: UiRichLinkTarget::parse("123456789").expect("fixture link is engine-local"),
            tooltip: None,
        });
        true
    }
}

#[test]
fn rich_parser_rejects_source_before_parse_or_cache_admission() {
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 4,
        max_output_bytes: 16,
        max_tokens: 8,
        max_token_bytes: 64,
        max_attributes_per_token: 4,
        max_attribute_bytes_per_token: 64,
        max_active_tag_depth: 8,
        ..RichParseBudget::default()
    });

    assert_eq!(
        parser.compile("12345", RichTextFormat::Plain),
        Err(RichTextParseError::SourceByteBudgetExceeded {
            actual_bytes: 5,
            max_bytes: 4,
        })
    );
}

#[test]
fn rich_parser_rejects_visible_text_expansion_before_range_projection() {
    let mut parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 64,
        max_output_bytes: 3,
        max_tokens: 8,
        max_token_bytes: 64,
        max_attributes_per_token: 4,
        max_attribute_bytes_per_token: 64,
        max_active_tag_depth: 8,
        ..RichParseBudget::default()
    });
    parser
        .register_emoji_shortcode("long", "a\u{301}\u{302}\u{303}")
        .expect("one grapheme replacement");

    assert_eq!(
        parser.parse(":long:", RichTextFormat::BbCodeV1),
        Err(RichTextParseError::OutputByteBudgetExceeded {
            attempted_bytes: 7,
            max_bytes: 3,
        })
    );
}

#[test]
fn rich_parser_rejects_inline_semantic_text_expansion_before_cache_publication() {
    let parser =
        RichTextParser::with_budget(RichParseBudget::default().with_max_semantic_text_bytes(4));

    assert_eq!(
        parser.compile(
            "<img src=\"res://icons/star.png\" alt=\"12345\">",
            RichTextFormat::HtmlSubsetV1,
        ),
        Err(RichTextParseError::SemanticTextByteBudgetExceeded {
            attempted_bytes: 5,
            max_bytes: 4,
        })
    );
}

#[test]
fn rich_parser_rejects_active_tag_depth_before_stack_growth() {
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 64,
        max_output_bytes: 64,
        max_tokens: 8,
        max_token_bytes: 64,
        max_attributes_per_token: 4,
        max_attribute_bytes_per_token: 64,
        max_active_tag_depth: 2,
        ..RichParseBudget::default()
    });

    for (markup, format) in [
        ("<b><i><u>x</u></i></b>", RichTextFormat::HtmlSubsetV1),
        ("[b][i][u]x[/u][/i][/b]", RichTextFormat::BbCodeV1),
    ] {
        assert_eq!(
            parser.parse(markup, format),
            Err(RichTextParseError::ActiveTagDepthBudgetExceeded {
                attempted_depth: 3,
                max_depth: 2,
            })
        );
    }
}

#[test]
fn rich_parser_rejects_markup_tokens_before_style_or_decorator_dispatch() {
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 64,
        max_output_bytes: 64,
        max_tokens: 2,
        max_token_bytes: 64,
        max_attributes_per_token: 4,
        max_attribute_bytes_per_token: 64,
        max_active_tag_depth: 8,
        ..RichParseBudget::default()
    });

    for (markup, format) in [
        ("<b></b><i>x</i>", RichTextFormat::HtmlSubsetV1),
        ("[b][/b][i]x[/i]", RichTextFormat::BbCodeV1),
    ] {
        assert_eq!(
            parser.parse(markup, format),
            Err(RichTextParseError::TokenBudgetExceeded {
                attempted_tokens: 3,
                max_tokens: 2,
            })
        );
    }

    assert_eq!(
        parser.parse("**a** *b*", RichTextFormat::MarkdownInlineV1),
        Err(RichTextParseError::TokenBudgetExceeded {
            attempted_tokens: 4,
            max_tokens: 2,
        })
    );
}

#[test]
fn rich_parser_rejects_attribute_count_and_bytes_before_string_allocation() {
    let count_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_tokens: 8,
        max_token_bytes: 64,
        max_attributes_per_token: 1,
        max_attribute_bytes_per_token: 64,
        max_active_tag_depth: 8,
        ..RichParseBudget::default()
    });
    for (markup, format) in [
        ("<span a='1' b='2'>x</span>", RichTextFormat::HtmlSubsetV1),
        ("[p a=1 b=2]x[/p]", RichTextFormat::BbCodeV1),
    ] {
        assert_eq!(
            count_parser.parse(markup, format),
            Err(RichTextParseError::AttributeCountBudgetExceeded {
                attempted_attributes: 2,
                max_attributes: 1,
            })
        );
    }

    let byte_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_tokens: 8,
        max_token_bytes: 64,
        max_attributes_per_token: 4,
        max_attribute_bytes_per_token: 3,
        max_active_tag_depth: 8,
        ..RichParseBudget::default()
    });
    for (markup, format) in [
        ("<span a='1234'>x</span>", RichTextFormat::HtmlSubsetV1),
        ("[p a=1234]x[/p]", RichTextFormat::BbCodeV1),
    ] {
        assert_eq!(
            byte_parser.parse(markup, format),
            Err(RichTextParseError::AttributeByteBudgetExceeded {
                attempted_bytes: 5,
                max_bytes: 3,
            })
        );
    }
    assert_eq!(
        byte_parser.parse("[color=1234]x[/color]", RichTextFormat::BbCodeV1),
        Err(RichTextParseError::AttributeByteBudgetExceeded {
            attempted_bytes: 4,
            max_bytes: 3,
        })
    );
}

#[test]
fn rich_parser_rejects_oversized_token_before_tag_name_allocation() {
    let parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_tokens: 8,
        max_token_bytes: 9,
        max_attributes_per_token: 4,
        max_attribute_bytes_per_token: 64,
        max_active_tag_depth: 8,
        ..RichParseBudget::default()
    });

    for (markup, format) in [
        ("<abcdefgh>", RichTextFormat::HtmlSubsetV1),
        ("[abcdefgh]", RichTextFormat::BbCodeV1),
    ] {
        assert_eq!(
            parser.parse(markup, format),
            Err(RichTextParseError::TokenByteBudgetExceeded {
                attempted_bytes: 10,
                max_bytes: 9,
            })
        );
    }
}

#[test]
fn rich_parser_rejects_run_paragraph_table_and_cell_growth_before_publish() {
    let run_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_runs: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        run_parser.parse("[b]a[/b]x", RichTextFormat::BbCodeV1),
        Err(RichTextParseError::RunCountBudgetExceeded {
            attempted_runs: 2,
            max_runs: 1,
        })
    );

    let paragraph_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_paragraphs: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        paragraph_parser.parse("[p]a[/p][p]b[/p]", RichTextFormat::BbCodeV1),
        Err(RichTextParseError::ParagraphCountBudgetExceeded {
            attempted_paragraphs: 2,
            max_paragraphs: 1,
        })
    );

    let table_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_tables: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        table_parser.parse(
            "[table][cell]a[/cell][/table][table][cell]b[/cell][/table]",
            RichTextFormat::BbCodeV1,
        ),
        Err(RichTextParseError::TableCountBudgetExceeded {
            attempted_tables: 2,
            max_tables: 1,
        })
    );

    let cell_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_table_cells: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        cell_parser.parse(
            "[table=2][cell]a[/cell][cell]b[/cell][/table]",
            RichTextFormat::BbCodeV1,
        ),
        Err(RichTextParseError::TableCellCountBudgetExceeded {
            attempted_cells: 2,
            max_cells: 1,
        })
    );
}

#[test]
fn rich_parser_rejects_projection_index_growth_before_compiled_publish() {
    let parser = RichTextParser::with_budget(
        RichParseBudget::new(128, 64).with_representation_limits(16, 16, 4, 16, 1),
    );

    assert_eq!(
        parser.parse(
            "[table=2][cell]a[/cell][cell]b[/cell][/table]",
            RichTextFormat::BbCodeV1,
        ),
        Err(RichTextParseError::ProjectionIndexBudgetExceeded {
            attempted_indices: 2,
            max_indices: 1,
        })
    );
}

#[test]
fn rich_parser_rejects_block_and_table_depth_before_stack_growth() {
    let block_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 128,
        max_output_bytes: 64,
        max_block_depth: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        block_parser.parse("[ul][ul][li]x[/li][/ul][/ul]", RichTextFormat::BbCodeV1,),
        Err(RichTextParseError::BlockDepthBudgetExceeded {
            attempted_depth: 2,
            max_depth: 1,
        })
    );

    let table_parser = RichTextParser::with_budget(RichParseBudget {
        max_source_bytes: 192,
        max_output_bytes: 64,
        max_table_depth: 1,
        ..RichParseBudget::default()
    });
    assert_eq!(
        table_parser.parse(
            "[table][cell][table][cell]x[/cell][/table][/cell][/table]",
            RichTextFormat::BbCodeV1,
        ),
        Err(RichTextParseError::TableDepthBudgetExceeded {
            attempted_depth: 2,
            max_depth: 1,
        })
    );
}

#[test]
fn rich_parser_isolates_decorator_panics_as_typed_failure() {
    let mut parser = RichTextParser::default();
    parser
        .register_decorator(PanickingDecorator)
        .expect("panic test decorator registration");

    assert_eq!(
        parser.parse("[panic]x[/panic]", RichTextFormat::BbCodeV1),
        Err(RichTextParseError::DecoratorPanicked {
            tag: "panic".to_string(),
        })
    );
    assert_eq!(
        parser
            .parse("plain", RichTextFormat::Plain)
            .expect("a decorator panic must not poison the parser")
            .text
            .as_ref(),
        "plain"
    );
}

#[test]
fn rich_parser_budgets_decorator_and_retained_run_metadata() {
    let mut decorator_parser = RichTextParser::with_budget(RichParseBudget {
        max_decorator_metadata_bytes_per_call: 8,
        ..RichParseBudget::default()
    });
    decorator_parser
        .register_decorator(OversizedMetadataDecorator)
        .expect("metadata test decorator registration");
    assert_eq!(
        decorator_parser.parse("[oversized]x[/oversized]", RichTextFormat::BbCodeV1),
        Err(RichTextParseError::DecoratorMetadataBudgetExceeded {
            tag: "oversized".to_string(),
            attempted_bytes: 9,
            max_bytes: 8,
        })
    );

    let retained_parser = RichTextParser::with_budget(RichParseBudget {
        max_retained_run_metadata_bytes: 10,
        ..RichParseBudget::default()
    });
    assert_eq!(
        retained_parser.parse(
            "[url=123456]a[/url][url=abcdef]b[/url]",
            RichTextFormat::BbCodeV1,
        ),
        Err(RichTextParseError::RunMetadataBudgetExceeded {
            attempted_bytes: 12,
            max_bytes: 10,
        })
    );
}
