use super::*;

#[test]
fn text_rich_markdown_ui_adapter_uses_stripped_text_ranges() {
    let parsed = parse_source_text("before **bold** after", RichTextFormat::Markdown);

    assert_eq!(parsed.text(), "before bold after");
    assert_eq!(parsed.runs.len(), 3);
    assert_eq!(
        parsed.runs[0].source_range,
        UiTextRange { start: 0, end: 7 }
    );
    assert_eq!(parsed.runs[1].kind, UiTextRunKind::Strong);
    assert_eq!(parsed.runs[1].text(), "bold");
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

    assert_eq!(parsed.text(), "Help\u{fffc}");
    assert_eq!(parsed.runs.len(), 2);
    assert_eq!(parsed.runs[0].kind, UiTextRunKind::Link);
    assert_eq!(
        parsed.runs[0].link().map(|link| link.href.as_str()),
        Some("res://docs/help.md")
    );
    assert!(matches!(
        parsed.runs[1].inline(),
        Some(InlineObjectRef::Image { size, .. }) if size.to_array() == [18.0, 20.0]
    ));
    assert_eq!(parsed.rich.link_runs().count(), 1);
    assert_eq!(parsed.rich.inline_runs().count(), 1);
    assert_eq!(parsed.rich.resource_ids().len(), 1);
    assert_eq!(
        parsed
            .rich
            .run_for_range(0, 4)
            .and_then(|run| run.link.as_ref())
            .map(|link| link.href.as_str()),
        Some("res://docs/help.md")
    );
    assert_eq!(
        parsed.rich.cluster_ranges(),
        &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 7)]
    );
}

#[test]
fn text_rich_ui_adapter_reuses_compiled_source_without_run_substrings() {
    let first = parse_source_text("before **bold** after", RichTextFormat::Markdown);
    let repeated = parse_source_text("before **bold** after", RichTextFormat::Markdown);
    let bold = &first.runs[1];

    assert!(std::sync::Arc::ptr_eq(&first.rich, &repeated.rich));
    assert_eq!(bold.text(), "bold");
    assert_eq!(
        bold.text().as_ptr(),
        first.text()[bold.source_range.start..].as_ptr()
    );
    assert!(std::ptr::eq(
        bold.style(),
        &first.rich.parsed().runs[1].style
    ));
}

#[test]
fn rich_layout_projection_exposes_ordered_non_overlapping_local_runs() {
    use crate::text::layout::RichTextLayoutSource;

    let parsed = parse_source_text(
        "pre <b>bold</b> <i>italic</i> <a href=\"res://docs/link.md\">link</a> post",
        RichTextFormat::Html,
    );
    let projection = parsed.project_range(2..18, None);
    let runs = (0..projection.run_count())
        .map(|index| projection.run(index).expect("projected rich run"))
        .collect::<Vec<_>>();

    assert!(runs.iter().all(|run| {
        run.byte_range.0 <= run.byte_range.1 && run.byte_range.1 as usize <= projection.text().len()
    }));
    assert!(runs.windows(2).all(|pair| {
        pair[0].byte_range.1 <= pair[1].byte_range.0 && pair[0].source_index < pair[1].source_index
    }));
}

#[test]
fn rich_layout_artifact_retains_semantics_after_compiled_cache_eviction() {
    use zircon_runtime_interface::ui::{
        layout::UiFrame,
        surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
    };

    let markup = "before <a href=\"res://docs/help.md\">help</a> after";
    let mut style = UiResolvedStyle::default();
    style.rich_text_format = RichTextFormat::Html.into();
    style.wrap = UiTextWrap::None;
    style.text_overflow = UiTextOverflow::Clip;
    let layout = crate::ui::text::layout_engine::layout_text(
        markup,
        &style,
        UiFrame::new(0.0, 0.0, 320.0, 40.0),
        None,
    );
    let handle = layout
        .rich_text_artifact
        .as_ref()
        .expect("rich layout should carry a compiled artifact handle");
    let before_eviction = crate::text::resolve_compiled_rich_text_artifact(handle)
        .expect("layout handle should resolve its compiled artifact");

    for index in 0..300 {
        let _ = parse_source_text(
            &format!("<a href=\"res://docs/{index}.md\">entry {index}</a>"),
            RichTextFormat::Html,
        );
    }

    assert!(
        crate::text::rich::lookup_compiled_rich_text(markup, RichTextFormat::Html).is_none(),
        "the bounded parser cache should evict the original entry under pressure"
    );
    let after_eviction = crate::text::resolve_compiled_rich_text_artifact(handle)
        .expect("active layout must retain compiled rich-text semantics after cache eviction");
    assert!(std::sync::Arc::ptr_eq(&before_eviction, &after_eviction));
    assert_eq!(
        after_eviction
            .link_runs()
            .next()
            .and_then(|run| run.link.as_ref())
            .map(|link| link.href.as_str()),
        Some("res://docs/help.md")
    );
}

#[test]
fn render_prepare_preserves_the_layout_rich_artifact_after_cache_eviction() {
    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::{UiFrame, UiPoint},
        surface::{
            UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiTextOverflow, UiTextWrap,
        },
    };

    let markup = "before <a href=\"res://docs/help.md\">help</a> after";
    let mut style = UiResolvedStyle::default();
    style.rich_text_format = RichTextFormat::Html.into();
    style.wrap = UiTextWrap::None;
    style.text_overflow = UiTextOverflow::Clip;
    let layout = crate::ui::text::layout_engine::layout_text(
        markup,
        &style,
        UiFrame::new(0.0, 0.0, 320.0, 40.0),
        None,
    );
    let original_handle = layout
        .rich_text_artifact
        .as_ref()
        .expect("rich layout should retain its compiled artifact")
        .clone();
    let original = crate::text::resolve_compiled_rich_text_artifact(&original_handle)
        .expect("layout artifact should resolve before extract");
    let mut commands = vec![UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 320.0, 40.0),
        clip_frame: None,
        z_index: 0,
        style,
        text_layout: Some(layout),
        text: Some(markup.to_string()),
        image: None,
        opacity: 1.0,
    }];

    for index in 0..300 {
        let _ = parse_source_text(
            &format!("<a href=\"res://docs/{index}.md\">entry {index}</a>"),
            RichTextFormat::Html,
        );
    }
    assert!(crate::text::rich::lookup_compiled_rich_text(markup, RichTextFormat::Html).is_none());

    super::prepare_render_command_text_artifacts(&mut commands);

    let layout = commands[0]
        .text_layout
        .as_ref()
        .expect("prepared command layout");
    let prepared_handle = layout
        .rich_text_artifact
        .as_ref()
        .expect("extract must keep the layout-owned rich artifact");
    let prepared = crate::text::resolve_compiled_rich_text_artifact(prepared_handle)
        .expect("prepared command should resolve the layout-owned artifact");
    assert!(std::sync::Arc::ptr_eq(&original, &prepared));

    let link_start_x =
        layout.lines[0].frame.x + layout.lines[0].glyph_advances[..7].iter().sum::<f32>() + 0.1;
    assert_eq!(
        super::link_at_layout_point(
            layout,
            UiPoint::new(link_start_x, layout.lines[0].frame.y + 4.0),
        )
        .map(|hit| hit.href),
        Some("res://docs/help.md".to_string())
    );
}

#[test]
fn render_prepare_rebuilds_missing_or_stale_plain_glyph_artifacts() {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::UiFrame,
        surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    };

    let style = UiResolvedStyle::default();
    let mut layout = crate::ui::text::layout_engine::layout_text(
        "fi",
        &style,
        UiFrame::new(0.0, 0.0, 80.0, 24.0),
        None,
    );
    let original = crate::text::resolve_resolved_text_glyph_artifact(
        layout
            .rich_text_artifact
            .as_ref()
            .expect("plain layout artifact"),
    )
    .expect("plain layout must own glyph artifacts");
    let mut stale = (*original).clone();
    stale.font_generation = u64::MAX;
    layout.rich_text_artifact = Some(crate::text::register_resolved_text_glyph_artifact(
        Arc::new(stale),
    ));
    let mut commands = vec![UiRenderCommand {
        node_id: UiNodeId::new(91),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 80.0, 24.0),
        clip_frame: None,
        z_index: 0,
        style,
        text_layout: Some(layout),
        text: Some("fi".to_string()),
        image: None,
        opacity: 1.0,
    }];

    super::prepare_render_command_text_artifacts(&mut commands);

    let artifact = crate::text::resolve_resolved_text_glyph_artifact(
        commands[0]
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
            .expect("extract must replace stale plain artifacts"),
    )
    .expect("extract must keep a plain glyph artifact");
    assert_eq!(
        artifact.font_generation,
        crate::text::font::shared_font_database_generation()
    );
}

#[test]
fn render_prepare_rebuilds_writing_mode_mismatched_plain_glyph_artifact() {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::UiFrame,
        surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiTextWritingMode},
    };

    let style = UiResolvedStyle::default();
    let mut layout = crate::ui::text::layout_engine::layout_text(
        "fi",
        &style,
        UiFrame::new(0.0, 0.0, 80.0, 24.0),
        None,
    );
    let original = crate::text::resolve_resolved_text_glyph_artifact(
        layout
            .rich_text_artifact
            .as_ref()
            .expect("plain layout artifact"),
    )
    .expect("plain layout must own glyph artifacts");
    let mut stale = (*original).clone();
    stale.writing_mode = UiTextWritingMode::VerticalRl;
    layout.rich_text_artifact = Some(crate::text::register_resolved_text_glyph_artifact(
        Arc::new(stale),
    ));
    let mut commands = vec![UiRenderCommand {
        node_id: UiNodeId::new(92),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 80.0, 24.0),
        clip_frame: None,
        z_index: 0,
        style,
        text_layout: Some(layout),
        text: Some("fi".to_string()),
        image: None,
        opacity: 1.0,
    }];

    super::prepare_render_command_text_artifacts(&mut commands);

    let artifact = crate::text::resolve_resolved_text_glyph_artifact(
        commands[0]
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
            .expect("prepare must replace writing-mode-mismatched plain artifacts"),
    )
    .expect("prepare must retain a plain glyph artifact");
    assert_eq!(artifact.writing_mode, UiTextWritingMode::HorizontalTb);
}

#[test]
fn render_prepare_reuses_current_plain_glyph_artifact() {
    use crate::ui::text::{UiTextLayoutRequest, UiTextMeasureCache};
    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::UiFrame,
        surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    };

    let style = UiResolvedStyle::default();
    let frame = UiFrame::new(0.0, 0.0, 80.0, 24.0);
    let request = UiTextLayoutRequest::new("fi", &style, frame, None);
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();
    let first_layout = cache.resolve_or_shape(&request).layout;
    let original = crate::text::resolve_resolved_text_glyph_artifact(
        first_layout
            .rich_text_artifact
            .as_ref()
            .expect("plain layout artifact"),
    )
    .expect("plain layout must own glyph artifacts");
    cache.begin_frame();
    let layout = cache.resolve_or_shape(&request).layout;
    let mut commands = vec![UiRenderCommand {
        node_id: UiNodeId::new(93),
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: None,
        z_index: 0,
        style,
        text_layout: Some(layout),
        text: Some("fi".to_string()),
        image: None,
        opacity: 1.0,
    }];

    super::prepare_render_command_text_artifacts(&mut commands);

    let prepared = crate::text::resolve_resolved_text_glyph_artifact(
        commands[0]
            .text_layout
            .as_ref()
            .and_then(|layout| layout.rich_text_artifact.as_ref())
            .expect("prepare must retain a current plain glyph artifact"),
    )
    .expect("prepared command must resolve its current artifact");
    assert!(std::sync::Arc::ptr_eq(&original, &prepared));
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

    let hit = super::link_at_layout_point(&layout, point).unwrap_or_else(|| {
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

    let hit = super::link_at_layout_point(&layout, point)
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
        super::link_at_layout_point(&layout, padding_point).is_none(),
        "physical cell padding/background must not become an implicit link target"
    );
}
