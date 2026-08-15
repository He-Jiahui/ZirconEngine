use super::{
    pending_owner_text_request, prewarm_render_command_text,
    resolve_missing_render_command_text_layouts, ui_text_shape_prewarm_pool,
    PendingOwnerTextLayout, PendingOwnerTextLayouts,
};
use crate::core::runtime::tasks::TaskPools;
use crate::text::{TextDocumentKey, TEXT_SHAPING_RUN_MAX_BYTES};
use crate::ui::text::{UiTextMeasureCache, UiTextViewport};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiEditableTextState, UiRenderCommand, UiRenderCommandKind, UiResolvedStyle,
        UiRichTextFormat, UiTextWritingMode,
    },
};

#[test]
fn prewarm_render_command_text_projects_rich_and_vertical_layout_spans() {
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(
        &[
            text_command(
                "**sample base.zui**",
                UiResolvedStyle {
                    rich_text_format: UiRichTextFormat::Markdown,
                    font_size: 10.0,
                    line_height: 12.0,
                    ..UiResolvedStyle::default()
                },
            ),
            text_command(
                "folder-open-outline.svg",
                UiResolvedStyle {
                    text_writing_mode: UiTextWritingMode::VerticalRl,
                    font_size: 10.0,
                    line_height: 12.0,
                    ..UiResolvedStyle::default()
                },
            ),
            text_command(
                "**sample base.zui**",
                UiResolvedStyle {
                    rich_text_format: UiRichTextFormat::Markdown,
                    font_size: 10.0,
                    line_height: 12.0,
                    ..UiResolvedStyle::default()
                },
            ),
        ],
        &PendingOwnerTextLayouts::default(),
        &mut cache,
    );

    let report = cache.frame_shape_prewarm_report();
    assert_eq!(report.requested_count, 3);
    assert_eq!(report.cache_miss_count, 2);
    assert_eq!(report.batch_duplicate_count, 1);
    assert_eq!(report.shaped_count, 2);
}

#[test]
fn rich_and_vertical_prewarm_match_normal_layout_source_without_inline_objects() {
    let rich_style = UiResolvedStyle {
        rich_text_format: UiRichTextFormat::Markdown,
        font_weight: 700,
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let mut vertical_style = rich_style.clone();
    vertical_style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let mut commands = vec![
        text_command("plain **bold**", rich_style),
        text_command("plain **bold**", vertical_style),
    ];
    let pending = PendingOwnerTextLayouts::default();
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);
    let prewarm = cache.frame_shape_prewarm_report();
    let before_layout = cache.frame_shaped_run_report();
    resolve_missing_render_command_text_layouts(&mut commands, &pending, Some(&mut cache));
    let after_layout = cache.frame_shaped_run_report();

    assert_eq!(prewarm.requested_count, 2);
    assert_eq!(prewarm.cache_miss_count, 2);
    assert_eq!(prewarm.shaped_count, 2);
    assert!(
        after_layout.hit_count >= before_layout.hit_count.saturating_add(2),
        "rich and vertical layout must consume base-style prewarm entries when no inline object is present"
    );
}

#[test]
fn rich_inline_prewarm_coalesces_effectively_equal_adjacent_spans() {
    let style = UiResolvedStyle {
        rich_text_format: UiRichTextFormat::Html,
        font_weight: 700,
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let mut commands = vec![text_command(
        "plain<b>bold</b><img src=\"res://icons/star.png\" width=\"16\" height=\"16\">tail",
        style,
    )];
    let pending = PendingOwnerTextLayouts::default();
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);
    let prewarm = cache.frame_shape_prewarm_report();
    let before_layout = cache.frame_shaped_run_report();
    resolve_missing_render_command_text_layouts(&mut commands, &pending, Some(&mut cache));
    let after_layout = cache.frame_shaped_run_report();
    assert_eq!(prewarm.requested_count, 2);
    assert_eq!(prewarm.cache_miss_count, 2);
    assert_eq!(prewarm.shaped_count, 2);
    assert!(
        after_layout.hit_count >= before_layout.hit_count.saturating_add(2),
        "inline rich layout must consume the coalesced resolved-span prewarm entries"
    );
}

#[test]
fn rich_inline_prewarm_matches_canonical_hard_line_layout_requests() {
    let long_run = "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1);
    let text = format!(
        "first\r\nsecond\u{2028}{long_run}<img src=\"res://icons/star.png\" width=\"16\" height=\"16\">tail"
    );
    let style = UiResolvedStyle {
        rich_text_format: UiRichTextFormat::Html,
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let mut commands = vec![text_command(&text, style)];
    let pending = PendingOwnerTextLayouts::default();
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);
    let prewarm = cache.frame_shape_prewarm_report();
    let before_layout = cache.frame_shaped_run_report();
    resolve_missing_render_command_text_layouts(&mut commands, &pending, Some(&mut cache));
    let after_layout = cache.frame_shaped_run_report();

    assert_eq!(prewarm.requested_count, 5);
    assert_eq!(prewarm.cache_miss_count, 5);
    assert_eq!(prewarm.shaped_count, 5);
    assert!(
        after_layout.hit_count >= before_layout.hit_count.saturating_add(5),
        "inline rich layout must reuse the canonical hard-line prewarm entries"
    );
}

#[test]
fn normal_rich_prewarm_matches_markup_joined_shape_cap() {
    let first = "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES / 2);
    let second = "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES / 2 + 1);
    let text = format!("{first}<b>{second}</b>");
    let style = UiResolvedStyle {
        rich_text_format: UiRichTextFormat::Html,
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let mut commands = vec![text_command(&text, style)];
    let pending = PendingOwnerTextLayouts::default();
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);
    let prewarm = cache.frame_shape_prewarm_report();
    let before_layout = cache.frame_shaped_run_report();
    resolve_missing_render_command_text_layouts(&mut commands, &pending, Some(&mut cache));
    let after_layout = cache.frame_shaped_run_report();
    let layout = commands[0]
        .text_layout
        .as_ref()
        .expect("normal rich text should resolve after prewarm");

    assert_eq!(prewarm.requested_count, 2);
    assert_eq!(prewarm.cache_miss_count, 2);
    assert_eq!(prewarm.shaped_count, 2);
    assert_eq!(layout.lines.len(), 2);
    assert!(
        after_layout.hit_count >= before_layout.hit_count.saturating_add(2),
        "normal rich layout must reuse prewarmed requests after markup runs join at the cap"
    );
}

#[test]
fn vertical_prewarm_preserves_unicode_hard_lines_and_run_caps() {
    let vertical_style = UiResolvedStyle {
        text_writing_mode: UiTextWritingMode::VerticalRl,
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = format!(
        "first\r\nsecond\u{2028}{}",
        "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1)
    );
    let commands = vec![text_command(&text, vertical_style)];
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &PendingOwnerTextLayouts::default(), &mut cache);

    let prewarm = cache.frame_shape_prewarm_report();
    assert_eq!(
        prewarm.requested_count, 4,
        "CRLF, Unicode separators, and the shaping cap must split vertical prewarm requests exactly as layout does"
    );
    assert_eq!(prewarm.cache_miss_count, 4);
    assert_eq!(prewarm.shaped_count, 4);
}

#[test]
fn owner_text_prewarm_overlap_uses_shared_compute_pool_and_one_scoped_join() {
    let prewarm_pool = ui_text_shape_prewarm_pool();
    let process_pools = TaskPools::process_default();
    let extract_source = include_str!("../extract.rs");

    assert!(prewarm_pool.shares_execution_owner_with(process_pools.compute()));
    assert_eq!(extract_source.matches("pool.in_place_scope").count(), 1);
    assert_eq!(
        extract_source
            .matches("prewarm_owner_text_requests(&requests, cache)")
            .count(),
        1
    );
    assert!(!extract_source.contains("spawn_named_thread"));
}

#[test]
fn pending_owner_request_retains_document_key_and_viewport() {
    let command = text_command("retained viewport", UiResolvedStyle::default());
    let document_key = TextDocumentKey::new(42, 7);
    let viewport = UiTextViewport::new(12.0, 24.0, 2).expect("finite viewport");
    let pending = PendingOwnerTextLayout {
        command_index: 0,
        document_key,
        viewport: Some(viewport),
        editable: None,
    };

    let request = pending_owner_text_request(&command, &pending)
        .expect("pending owner command should retain source text");

    assert_eq!(request.document_key, Some(document_key));
    assert_eq!(request.viewport, Some(viewport));
    assert_eq!(request.text, "retained viewport");
}

#[test]
fn pending_owner_resolution_preserves_editable_empty_text_without_prewarm() {
    let editable = UiEditableTextState {
        text: String::new(),
        ..UiEditableTextState::default()
    };
    let mut commands = vec![text_command("", UiResolvedStyle::default())];
    let mut pending = PendingOwnerTextLayouts::default();
    pending.push(0, TextDocumentKey::new(9, 3), None, Some(editable.clone()));
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);
    assert_eq!(cache.frame_shape_prewarm_report().requested_count, 0);
    resolve_missing_render_command_text_layouts(&mut commands, &pending, Some(&mut cache));

    let layout = commands[0]
        .text_layout
        .as_ref()
        .expect("empty owner text must resolve after the prewarm filter");
    assert_eq!(layout.editable.as_ref(), Some(&editable));
}

#[test]
fn short_clipped_plain_owner_prewarm_remains_batched() {
    let style = UiResolvedStyle {
        wrap: zircon_runtime_interface::ui::surface::UiTextWrap::None,
        text_overflow: zircon_runtime_interface::ui::surface::UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let mut command = text_command("ordinary owner", style);
    command.frame = UiFrame::new(0.0, 0.0, 180.0, 200.0);
    command.clip_frame = Some(UiFrame::new(0.0, 0.0, 180.0, 20.0));
    let commands = vec![command];
    let mut pending = PendingOwnerTextLayouts::default();
    pending.push(
        0,
        TextDocumentKey::new(72, 1),
        Some(UiTextViewport::new(0.0, 20.0, 2).expect("finite document viewport")),
        None,
    );
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);

    assert_eq!(cache.frame_shape_prewarm_report().requested_count, 1);
}

#[test]
fn non_virtualizable_owner_prewarm_remains_batched() {
    let style = UiResolvedStyle {
        wrap: zircon_runtime_interface::ui::surface::UiTextWrap::Glyph,
        text_overflow: zircon_runtime_interface::ui::surface::UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let mut command = text_command("wrapped owner", style);
    command.frame = UiFrame::new(0.0, 0.0, 180.0, 200.0);
    command.clip_frame = Some(UiFrame::new(0.0, 0.0, 180.0, 20.0));
    let commands = vec![command];
    let mut pending = PendingOwnerTextLayouts::default();
    pending.push(
        0,
        TextDocumentKey::new(72, 2),
        Some(UiTextViewport::new(0.0, 20.0, 2).expect("finite document viewport")),
        None,
    );
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);

    assert_eq!(cache.frame_shape_prewarm_report().requested_count, 1);
}

#[test]
fn oversized_single_line_owner_prewarm_remains_batched_when_viewport_covers_it() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 20.0,
        wrap: zircon_runtime_interface::ui::surface::UiTextWrap::None,
        text_overflow: zircon_runtime_interface::ui::surface::UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let text = "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1);
    let mut command = text_command(&text, style);
    command.frame = UiFrame::new(0.0, 0.0, 180.0, 20.0);
    command.clip_frame = Some(UiFrame::new(0.0, 0.0, 180.0, 20.0));
    let commands = vec![command];
    let mut pending = PendingOwnerTextLayouts::default();
    pending.push(
        0,
        TextDocumentKey::new(72, 3),
        Some(UiTextViewport::new(0.0, 20.0, 2).expect("finite document viewport")),
        None,
    );
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);

    assert!(
        cache.frame_shape_prewarm_report().requested_count > 0,
        "a viewport that covers every capped hard line must retain full-source prewarm"
    );
}

#[test]
fn viewported_owner_prewarm_defers_shaping_to_the_visible_window() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 20.0,
        wrap: zircon_runtime_interface::ui::surface::UiTextWrap::None,
        text_overflow: zircon_runtime_interface::ui::surface::UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let text = (0..10_000)
        .map(|index| format!("log-row-{index:05}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut command = text_command(&text, style);
    command.frame = UiFrame::new(0.0, 0.0, 240.0, 200_000.0);
    command.clip_frame = Some(UiFrame::new(0.0, 241.0, 240.0, 8.0));
    let mut commands = vec![command];
    let mut pending = PendingOwnerTextLayouts::default();
    pending.push(
        0,
        TextDocumentKey::new(73, 1),
        Some(UiTextViewport::new(241.0, 8.0, 2).expect("finite document viewport")),
        None,
    );
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();

    prewarm_render_command_text(&commands, &pending, &mut cache);
    assert_eq!(
        cache.frame_shape_prewarm_report().requested_count,
        0,
        "a viewport owner must not prewarm all 10,000 source paragraphs"
    );

    let before_layout = cache.frame_shaped_run_report();
    resolve_missing_render_command_text_layouts(&mut commands, &pending, Some(&mut cache));
    let after_layout = cache.frame_shaped_run_report();
    let layout = commands[0]
        .text_layout
        .as_ref()
        .expect("owner text must resolve after the deferred prewarm");

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "log-row-00012");
    assert_eq!(layout.measured_height, 200_000.0);
    assert_eq!(
        after_layout
            .miss_count
            .saturating_sub(before_layout.miss_count),
        6,
        "layout must shape only the visible line and fixed metrics"
    );
}

fn text_command(text: &str, style: UiResolvedStyle) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 180.0, 20.0),
        clip_frame: None,
        z_index: 0,
        style,
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}
