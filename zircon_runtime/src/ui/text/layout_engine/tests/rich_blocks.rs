use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextWrap,
        UiTextWritingMode,
    },
};

use super::{layout_text, test_style};

#[test]
fn text_rich_bbcode_indent_reduces_wrap_extent_and_insets_logical_start() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let frame = UiFrame::new(10.0, 0.0, 56.0, 120.0);

    let plain = layout_text("abcdefghij", &style, frame, None);
    let indented = layout_text("[indent]abcdefghij[/indent]", &style, frame, None);

    assert!(indented.lines.len() > plain.lines.len());
    assert!(indented.lines[0].frame.x > frame.x);
    assert!(
        indented
            .lines
            .iter()
            .all(|line| line.frame.right() <= frame.right() + 0.01)
    );
}

#[test]
fn text_rich_bbcode_list_wraps_continuation_with_hanging_prefix_indent() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let frame = UiFrame::new(0.0, 0.0, 74.0, 160.0);

    let layout = layout_text(
        "[ul][li]alpha beta gamma delta[/li][/ul]",
        &style,
        frame,
        None,
    );

    assert!(layout.lines.len() >= 2);
    assert!(layout.lines[0].text.starts_with("• "));
    assert!(layout.lines[1].frame.x > layout.lines[0].frame.x);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.frame.right() <= frame.right() + 0.01)
    );
}

#[test]
fn text_rich_bbcode_nested_list_adds_one_measured_indent_level() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;

    let layout = layout_text(
        "[ul][li]outer[ul][li]inner[/li][/ul][/li][/ul]",
        &style,
        UiFrame::new(0.0, 0.0, 240.0, 100.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert!(layout.lines[1].frame.x > layout.lines[0].frame.x);
}

#[test]
fn text_rich_bbcode_nested_list_hanging_indent_uses_the_inner_prefix() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
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
}

#[test]
fn text_rich_bbcode_paragraph_aligns_inside_its_inset_content_frame() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_align = UiTextAlign::Left;
    let frame = UiFrame::new(10.0, 0.0, 160.0, 40.0);

    let layout = layout_text(
        "[p align=center indent=24]centered[/p]",
        &style,
        frame,
        None,
    );

    let line = &layout.lines[0];
    assert!(line.frame.x > frame.x + 24.0);
    assert!(line.frame.right() < frame.right());
}

#[test]
fn text_rich_bbcode_rtl_indent_insets_the_logical_start_edge() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_direction = UiTextDirection::RightToLeft;
    style.text_align = UiTextAlign::Start;
    let frame = UiFrame::new(10.0, 0.0, 180.0, 40.0);

    let plain = layout_text("אבג", &style, frame, None);
    let indented = layout_text("[indent]אבג[/indent]", &style, frame, None);

    assert!(indented.lines[0].frame.right() < plain.lines[0].frame.right());
    assert!(indented.lines[0].frame.x >= frame.x - 0.01);
}

#[test]
fn text_rich_bbcode_deep_indent_saturates_inside_the_frame() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let markup = format!("{}x{}", "[indent]".repeat(48), "[/indent]".repeat(48));
    let frame = UiFrame::new(0.0, 0.0, 120.0, 80.0);

    let layout = layout_text(&markup, &style, frame, None);

    assert_eq!(layout.lines[0].text, "x");
    assert!(layout.lines[0].frame.x.is_finite());
    assert!(layout.lines[0].frame.x <= frame.right());
}

#[test]
fn text_rich_bbcode_vertical_first_indent_offsets_only_the_first_column() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(10.0, 20.0, 120.0, 52.0);

    let layout = layout_text("[p indent=18]甲乙丙丁戊己庚辛[/p]", &style, frame, None);

    assert!(layout.lines.len() >= 2);
    assert!(layout.lines[0].frame.y >= frame.y + 18.0 - 0.01);
    assert!((layout.lines[1].frame.y - frame.y).abs() <= 0.01);
    assert!(layout.lines[0].frame.x > layout.lines[1].frame.x);
}

#[test]
fn text_rich_bbcode_vertical_nested_indent_offsets_every_column() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(10.0, 20.0, 120.0, 52.0);

    let layout = layout_text("[indent]甲乙丙丁戊己庚辛[/indent]", &style, frame, None);

    assert!(layout.lines.len() >= 2);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.frame.y > frame.y + 0.01)
    );
}

#[test]
fn text_rich_bbcode_vertical_paragraph_alignment_uses_the_inline_axis() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(10.0, 20.0, 120.0, 120.0);

    let centered = layout_text("[p align=center]甲乙[/p]", &style, frame, None);
    let ended = layout_text("[p align=right]甲乙[/p]", &style, frame, None);
    let center = &centered.lines[0];
    let end = &ended.lines[0];

    assert!(center.frame.y > frame.y + 0.01);
    assert!(center.frame.bottom() < frame.bottom() - 0.01);
    assert!((end.frame.bottom() - frame.bottom()).abs() <= 0.01);
    assert!(end.frame.y > center.frame.y);
}
