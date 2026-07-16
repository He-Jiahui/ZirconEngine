use super::*;

#[test]
fn text_rich_markdown_ui_adapter_uses_stripped_text_ranges() {
    let parsed = parse_source_text("before **bold** after", RichTextFormat::Markdown);

    assert_eq!(parsed.text, "before bold after");
    assert_eq!(parsed.runs.len(), 3);
    assert_eq!(
        parsed.runs[0].source_range,
        UiTextRange { start: 0, end: 7 }
    );
    assert_eq!(parsed.runs[1].kind, UiTextRunKind::Strong);
    assert_eq!(parsed.runs[1].text, "bold");
    assert_eq!(
        parsed.runs[1].source_range,
        UiTextRange { start: 7, end: 11 }
    );
    assert_eq!(
        parsed.runs[2].source_range,
        UiTextRange { start: 11, end: 17 }
    );
}

#[test]
fn text_rich_html_ui_adapter_preserves_inline_and_link_metadata() {
    let parsed = parse_source_text(
        "<a href=\"res://docs/help.md\">Help</a><img src=\"res://icons/help.png\" width=\"18\" height=\"20\">",
        RichTextFormat::Html,
    );

    assert_eq!(parsed.text, "Help\u{fffc}");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].kind, UiTextRunKind::Link);
    assert_eq!(
        parsed.runs[0].link.as_ref().map(|link| link.href.as_str()),
        Some("res://docs/help.md")
    );
    assert!(matches!(
        parsed.runs[1].inline,
        Some(InlineObjectRef::Image { size, .. }) if size.to_array() == [18.0, 20.0]
    ));
}

#[test]
fn text_rich_link_hit_uses_upstream_affinity_at_run_end() {
    use zircon_runtime_interface::ui::{
        layout::{UiFrame, UiPoint},
        surface::{UiResolvedStyle, UiTextCaretAffinity, UiTextOverflow, UiTextWrap},
    };

    let mut style = UiResolvedStyle::default();
    style.rich_text_format = RichTextFormat::Html.into();
    style.wrap = UiTextWrap::None;
    style.text_overflow = UiTextOverflow::Clip;
    let layout = crate::ui::text::layout_engine::layout_text(
        "before <a href=\"res://docs/help.md\">help</a> after",
        &style,
        UiFrame::new(0.0, 0.0, 320.0, 40.0),
        None,
    );
    let link_end_x =
        layout.lines[0].frame.x + layout.lines[0].glyph_advances[..11].iter().sum::<f32>() - 0.1;

    let hit = super::link_at_layout_point(
        "before <a href=\"res://docs/help.md\">help</a> after",
        RichTextFormat::Html,
        &layout,
        UiPoint::new(link_end_x, layout.lines[0].frame.y + 4.0),
    )
    .expect("the trailing half of the final linked grapheme should activate the link");

    assert_eq!(hit.href, "res://docs/help.md");
    assert_eq!(hit.source_range, UiTextRange { start: 7, end: 11 });
    assert_eq!(hit.affinity, UiTextCaretAffinity::Upstream);
}

#[test]
fn text_rich_horizontal_table_link_hit_uses_the_containing_cell_line() {
    use zircon_runtime_interface::ui::{
        layout::{UiFrame, UiPoint},
        surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
    };

    let markup = "[table=2][cell]first[/cell][cell border=#73D7FF padding=18,12,16,10][url=res://docs/table-second.md]second link[/url][/cell][/table]";
    let mut style = UiResolvedStyle::default();
    style.rich_text_format = RichTextFormat::BbCode.into();
    style.wrap = UiTextWrap::None;
    style.text_overflow = UiTextOverflow::Clip;
    let layout = crate::ui::text::layout_engine::layout_text(
        markup,
        &style,
        UiFrame::new(10.0, 20.0, 360.0, 100.0),
        None,
    );
    let link_line = layout
        .lines
        .iter()
        .find(|line| line.text.contains("second link"))
        .expect("second table cell link line");
    let point = UiPoint::new(link_line.frame.x + 2.0, link_line.frame.y + 2.0);

    let hit = super::link_at_layout_point(markup, RichTextFormat::BbCode, &layout, point)
        .unwrap_or_else(|| {
            panic!(
                "the containing second-cell line must own the link hit; lines={:?}",
                layout.lines
            )
        });

    assert_eq!(hit.href, "res://docs/table-second.md");
}

#[test]
fn text_rich_vertical_table_link_hit_uses_the_containing_inline_slot() {
    use zircon_runtime_interface::ui::{
        layout::{UiFrame, UiPoint},
        surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap, UiTextWritingMode},
    };

    let markup = "[table=2][cell]上[/cell][cell border=#73D7FF padding=2,2,2,2][url=res://docs/vertical-cell.md]下[/url][/cell][/table]";
    let mut style = UiResolvedStyle::default();
    style.rich_text_format = RichTextFormat::BbCode.into();
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    style.wrap = UiTextWrap::None;
    style.text_overflow = UiTextOverflow::Clip;
    let layout = crate::ui::text::layout_engine::layout_text(
        markup,
        &style,
        UiFrame::new(10.0, 20.0, 220.0, 400.0),
        None,
    );
    let link_line = layout
        .lines
        .iter()
        .find(|line| line.text.contains('下'))
        .unwrap_or_else(|| {
            panic!(
                "lower vertical table link line; lines={:?}, boxes={:?}",
                layout.lines, layout.boxes
            )
        });
    let point = UiPoint::new(link_line.frame.x + 2.0, link_line.frame.y + 2.0);

    let hit = super::link_at_layout_point(markup, RichTextFormat::BbCode, &layout, point)
        .expect("the containing vertical inline slot must own the link hit");

    assert_eq!(hit.href, "res://docs/vertical-cell.md");
}

#[test]
fn text_rich_table_cell_padding_does_not_activate_its_link() {
    use zircon_runtime_interface::ui::{
        layout::{UiFrame, UiPoint},
        surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
    };

    let markup = "[table=1][cell border=#73D7FF bg=#102638 padding=24,20,18,16][url=res://docs/padded.md]linked[/url][/cell][/table]";
    let mut style = UiResolvedStyle::default();
    style.rich_text_format = RichTextFormat::BbCode.into();
    style.wrap = UiTextWrap::None;
    style.text_overflow = UiTextOverflow::Clip;
    let layout = crate::ui::text::layout_engine::layout_text(
        markup,
        &style,
        UiFrame::new(10.0, 20.0, 260.0, 100.0),
        None,
    );
    let cell = layout.boxes.first().expect("styled table cell box");
    let padding_point = UiPoint::new(cell.frame.x + 2.0, cell.frame.y + 2.0);

    assert!(
        super::link_at_layout_point(markup, RichTextFormat::BbCode, &layout, padding_point,)
            .is_none(),
        "physical cell padding/background must not become an implicit link target"
    );
}
