use crate::text::{
    RichListItemKind, RichOrderedListMarker, RichParseResult, RichTextFormat, TextAlign,
};

use super::parser_registry::parse_rich_text as try_parse_rich_text;

fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    try_parse_rich_text(markup, format).expect("test rich source fits parser budgets")
}

#[test]
fn text_rich_bbcode_paragraph_attributes_use_block_metadata() {
    let parsed = parse_rich_text(
        "[p align=center indent=12]alpha[/p]tail",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "alpha\ntail");
    let (_, paragraph) = parsed
        .paragraphs
        .iter()
        .find(|(range, _)| *range == (0, 5))
        .expect("paragraph metadata");
    assert_eq!(paragraph.align, Some(TextAlign::Center));
    assert_eq!(paragraph.indent, Some(12.0));
}

#[test]
fn text_rich_bbcode_indent_is_a_nested_logical_level() {
    let parsed = parse_rich_text(
        "[indent]outer [indent]inner[/indent][/indent]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "outer \ninner");
    assert_eq!(parsed.paragraphs.len(), 2);
    let inner_offset = parsed.text.find("inner").expect("inner text") as u32;
    assert_eq!(
        parsed
            .paragraphs
            .iter()
            .filter(|(range, paragraph)| {
                range.0 <= inner_offset
                    && inner_offset < range.1
                    && paragraph.indent_level == Some(1)
            })
            .count(),
        2
    );
}

#[test]
fn text_rich_bbcode_lists_emit_real_prefix_text_without_trailing_break() {
    let parsed = parse_rich_text(
        "[ul][li]one[/li][li]two[/li][/ul]tail",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "• one\n• two\ntail");
    let list_items = parsed
        .paragraphs
        .iter()
        .filter_map(|(_, paragraph)| paragraph.list_item.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(list_items.len(), 2);
    assert_eq!(list_items[0].marker_range, (0, 4));
    assert_eq!(list_items[1].marker_range, (8, 12));
    assert_eq!(list_items[0].level, 1);
    assert_eq!(list_items[0].kind, RichListItemKind::Unordered);
}

#[test]
fn text_rich_bbcode_nested_lists_preserve_order_and_depth_metadata() {
    let parsed = parse_rich_text(
        "[ol type=A][li]One[ul bullet=→][li]Inner[/li][/ul][/li][li]Two[/li][/ol]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "A. One\n→ Inner\nB. Two");
    let list_items = parsed
        .paragraphs
        .iter()
        .filter_map(|(_, paragraph)| paragraph.list_item.as_ref())
        .collect::<Vec<_>>();
    let prefixes = list_items
        .iter()
        .map(|item| &parsed.text[item.marker_range.0 as usize..item.marker_range.1 as usize])
        .collect::<Vec<_>>();
    assert_eq!(prefixes, vec!["A. ", "→ ", "B. "]);
    assert_eq!(
        list_items.iter().map(|item| item.level).collect::<Vec<_>>(),
        vec![1, 2, 1]
    );
    assert_eq!(
        list_items[0].kind,
        RichListItemKind::Ordered {
            ordinal: 1,
            marker: RichOrderedListMarker::AlphaUpper,
        }
    );
    assert_eq!(list_items[1].kind, RichListItemKind::Unordered);
    assert_eq!(
        list_items[2].kind,
        RichListItemKind::Ordered {
            ordinal: 2,
            marker: RichOrderedListMarker::AlphaUpper,
        }
    );
    let inner_offset = parsed.text.find("Inner").expect("inner item") as u32;
    assert_eq!(
        parsed
            .paragraphs
            .iter()
            .filter(|(range, paragraph)| {
                range.0 <= inner_offset
                    && inner_offset < range.1
                    && paragraph.indent_level == Some(1)
            })
            .count(),
        2
    );
}

#[test]
fn text_rich_bbcode_ordered_list_supports_alpha_and_roman_markers() {
    let parsed = parse_rich_text(
        "[ol type=a][li]alpha[/li][/ol][ol type=I][li]roman[/li][/ol]",
        RichTextFormat::BbCodeV1,
    );

    assert_eq!(parsed.text.as_ref(), "a. alpha\nI. roman");
}

#[test]
fn text_rich_bbcode_block_metadata_is_sorted_outer_before_inner() {
    let parsed = parse_rich_text(
        "[ul][li]outer[ul][li]inner[/li][/ul][/li][/ul]",
        RichTextFormat::BbCodeV1,
    );

    assert!(parsed.paragraphs.windows(2).all(|paragraphs| {
        let left = paragraphs[0].0;
        let right = paragraphs[1].0;
        left.0 < right.0 || (left.0 == right.0 && left.1 >= right.1)
    }));
}
