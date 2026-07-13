use crate::core::framework::render::RichTextFormat;
use zircon_runtime_interface::ui::surface::UiTextAlign;

use super::parse_rich_text;

#[test]
fn text_rich_bbcode_paragraph_attributes_use_block_metadata() {
    let parsed = parse_rich_text(
        "[p align=center indent=12]alpha[/p]tail",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "alpha\ntail");
    let (_, paragraph) = parsed
        .paragraphs
        .iter()
        .find(|(range, _)| *range == (0, 5))
        .expect("paragraph metadata");
    assert_eq!(paragraph.align, Some(UiTextAlign::Center));
    assert_eq!(paragraph.indent, Some(12.0));
}

#[test]
fn text_rich_bbcode_indent_is_a_nested_logical_level() {
    let parsed = parse_rich_text(
        "[indent]outer [indent]inner[/indent][/indent]",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "outer \ninner");
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
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "• one\n• two\ntail");
    let list_items = parsed
        .paragraphs
        .iter()
        .filter(|(_, paragraph)| paragraph.list_prefix.is_some())
        .collect::<Vec<_>>();
    assert_eq!(list_items.len(), 2);
    assert_eq!(list_items[0].1.list_prefix, Some((0, 4)));
    assert_eq!(list_items[1].1.list_prefix, Some((8, 12)));
}

#[test]
fn text_rich_bbcode_nested_lists_preserve_order_and_depth_metadata() {
    let parsed = parse_rich_text(
        "[ol type=A][li]One[ul bullet=→][li]Inner[/li][/ul][/li][li]Two[/li][/ol]",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "A. One\n→ Inner\nB. Two");
    let prefixes = parsed
        .paragraphs
        .iter()
        .filter_map(|(_, paragraph)| paragraph.list_prefix)
        .map(|range| &parsed.text[range.0 as usize..range.1 as usize])
        .collect::<Vec<_>>();
    assert_eq!(prefixes, vec!["A. ", "→ ", "B. "]);
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
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "a. alpha\nI. roman");
}

#[test]
fn text_rich_bbcode_block_metadata_is_sorted_outer_before_inner() {
    let parsed = parse_rich_text(
        "[ul][li]outer[ul][li]inner[/li][/ul][/li][/ul]",
        RichTextFormat::BbCode,
    );

    assert!(parsed.paragraphs.windows(2).all(|paragraphs| {
        let left = paragraphs[0].0;
        let right = paragraphs[1].0;
        left.0 < right.0 || (left.0 == right.0 && left.1 >= right.1)
    }));
}
