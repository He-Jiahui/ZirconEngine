#![cfg(feature = "ui")]

use zircon_runtime::text::{RichTextFormat, RichTextParser, TextAlign};
use zircon_runtime::ui::surface::layout_text;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedStyle, UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextWrap,
    },
};

fn block_style(wrap: UiTextWrap) -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap,
        text_overflow: UiTextOverflow::Clip,
        rich_text_format: UiRichTextFormat::BbCode,
        ..UiResolvedStyle::default()
    }
}

#[test]
fn runtime_text_rich_blocks_parse_paragraph_indent_and_nested_markers() {
    let parser = RichTextParser::default();
    let parsed = parser.parse(
        "[p align=center indent=12]title[/p][indent]body[/indent][ol type=A][li]One[ul bullet=→][li]Inner[/li][/ul][/li][li]Two[/li][/ol]",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "title\nbody\nA. One\n→ Inner\nB. Two");
    assert!(parsed.paragraphs.iter().any(|(_, paragraph)| {
        paragraph.align == Some(TextAlign::Center) && paragraph.indent == Some(12.0)
    }));
    assert_eq!(
        parsed
            .paragraphs
            .iter()
            .filter_map(|(_, paragraph)| paragraph.list_prefix)
            .map(|range| &parsed.text[range.0 as usize..range.1 as usize])
            .collect::<Vec<_>>(),
        vec!["A. ", "→ ", "B. "]
    );
}

#[test]
fn runtime_text_rich_blocks_layout_hanging_indent_and_inner_marker_width() {
    let style = block_style(UiTextWrap::WordSmart);
    let frame = UiFrame::new(0.0, 0.0, 180.0, 220.0);
    let layout_with = |inner_bullet: &str| {
        layout_text(
            &format!(
                "[ul bullet=O][li]outer[ul bullet={inner_bullet}][li]inner alpha beta gamma delta epsilon zeta eta theta[/li][/ul][/li][/ul]"
            ),
            &style,
            frame,
            None,
        )
    };

    let narrow = layout_with("•");
    let wide = layout_with("MMMM");
    let narrow_inner = narrow
        .lines
        .iter()
        .position(|line| line.text.starts_with("• inner"))
        .expect("narrow inner list first line");
    let wide_inner = wide
        .lines
        .iter()
        .position(|line| line.text.starts_with("MMMM inner"))
        .expect("wide inner list first line");

    assert!(narrow_inner + 1 < narrow.lines.len());
    assert!(wide_inner + 1 < wide.lines.len());
    assert!(wide.lines[wide_inner + 1].frame.x > narrow.lines[narrow_inner + 1].frame.x);
    assert!(wide
        .lines
        .iter()
        .all(|line| line.frame.right() <= frame.right() + 0.01));
}

#[test]
fn runtime_text_rich_blocks_layout_paragraph_alignment_and_rtl_logical_start() {
    let frame = UiFrame::new(10.0, 0.0, 180.0, 80.0);
    let mut centered_style = block_style(UiTextWrap::None);
    centered_style.text_align = UiTextAlign::Left;
    let centered = layout_text(
        "[p align=center indent=24]centered[/p]",
        &centered_style,
        frame,
        None,
    );
    assert!(centered.lines[0].frame.x > frame.x + 24.0);
    assert!(centered.lines[0].frame.right() < frame.right());

    let mut rtl_style = block_style(UiTextWrap::None);
    rtl_style.text_direction = UiTextDirection::RightToLeft;
    rtl_style.text_align = UiTextAlign::Start;
    let plain = layout_text("אבג", &rtl_style, frame, None);
    let indented = layout_text("[indent]אבג[/indent]", &rtl_style, frame, None);
    assert!(indented.lines[0].frame.right() < plain.lines[0].frame.right());
}

#[test]
fn runtime_text_rich_blocks_bound_hostile_nesting_to_finite_frames() {
    let markup = format!("{}x{}", "[indent]".repeat(48), "[/indent]".repeat(48));
    let frame = UiFrame::new(0.0, 0.0, 120.0, 80.0);
    let layout = layout_text(&markup, &block_style(UiTextWrap::Glyph), frame, None);

    assert_eq!(layout.lines[0].text, "x");
    assert!(layout.lines[0].frame.x.is_finite());
    assert!(layout.lines[0].frame.x <= frame.right());
}
