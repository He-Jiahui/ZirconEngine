use super::*;

#[test]
fn screen_space_ui_rich_inline_reuses_the_command_parse() {
    let source = include_str!("../rich_text.rs");
    let parse_call = ["parse_rich", "_text("].concat();

    assert_eq!(source.matches(&parse_call).count(), 1);
}

#[test]
fn screen_space_ui_plan_places_html_inline_image_without_placeholder_glyph() {
    let markup = "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\">after";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::HtmlSubsetV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 220.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(8),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(260, 100),
    );

    assert!(
        plan.vertices.is_empty(),
        "inline images must not fall back to a solid-color placeholder quad"
    );
    assert_eq!(plan.images.len(), 1);
    assert_eq!(
        plan.images[0].texture,
        crate::core::resource::ResourceId::from_stable_label("res://icons/star.png")
    );
    assert!((plan.images[0].frame.width - 16.0).abs() < 0.01);
    assert!((plan.images[0].frame.height - 24.0).abs() < 0.01);
    assert!(plan.images[0].frame.x > frame.x);
    assert_eq!(plan.native_texts.len(), 2);
    assert_eq!(plan.native_texts[0].text, "before");
    assert_eq!(plan.native_texts[1].text, "after");
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| !batch.text.contains('\u{fffc}'))
    );
}

#[test]
fn screen_space_ui_plan_renders_bbcode_icon_as_asset_batch() {
    let markup = "before[icon=res://icons/star.png|18x22|baseline|Favorite]after";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 18.0,
        line_height: 22.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCodeV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 240.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-icon"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(12),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(280, 100),
    );

    assert_eq!(plan.images.len(), 1);
    assert_eq!(
        plan.images[0].texture,
        crate::core::resource::ResourceId::from_stable_label("res://icons/star.png")
    );
    assert!((plan.images[0].frame.width - 18.0).abs() < 0.01);
    assert!((plan.images[0].frame.height - 22.0).abs() < 0.01);
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| !batch.text.contains('\u{fffc}'))
    );
    assert!(plan.vertices.is_empty());
}

#[test]
fn screen_space_ui_rich_font_size_override_keeps_the_resolved_line_baseline() {
    let markup = "[size=26]Large[/size] small";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCodeV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 240.0, 48.0);
    let layout = layout_text(markup, &style, frame, None);
    let expected_baseline = layout.lines[0].frame.y + layout.lines[0].baseline;
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-baseline"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(17),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(280, 100),
    );

    assert_eq!(plan.native_texts.len(), 2);
    assert!(
        plan.native_texts
            .iter()
            .any(|batch| batch.font_size == 26.0)
    );
    assert!(
        plan.native_texts
            .iter()
            .any(|batch| batch.font_size == 10.0)
    );
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| batch.glyph_artifact_line.is_some())
    );
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| { batch.text_decoration_baseline == Some(expected_baseline) })
    );
}

#[test]
fn screen_space_ui_plan_keeps_inline_image_retained_by_ellipsis() {
    let markup = "a<img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> trailing";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 24.0,
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Ellipsis,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::HtmlSubsetV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 34.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    assert!(layout.lines[0].ellipsized);

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-ellipsis"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(10),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(100, 80),
    );

    assert_eq!(plan.images.len(), 1);
    assert!((plan.images[0].frame.width - 16.0).abs() < 0.01);
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| !batch.text.contains('\u{fffc}'))
    );
}

#[test]
fn screen_space_ui_plan_places_rtl_inline_image_at_visual_run_offset() {
    let markup = "אב <img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> גד";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        text_direction: UiTextDirection::RightToLeft,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::HtmlSubsetV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 220.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let line = &layout.lines[0];
    let inline_run = line
        .runs
        .iter()
        .find(|run| run.text == "\u{fffc}")
        .expect("resolved visual inline run");
    let inline_visual_index = line.text[..inline_run.visual_range.start]
        .graphemes(true)
        .count();
    let expected_x = line.frame.x
        + line
            .glyph_advances
            .iter()
            .take(inline_visual_index)
            .sum::<f32>();
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-rtl"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(9),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(260, 100),
    );

    assert_eq!(plan.images.len(), 1);
    assert!((plan.images[0].frame.x - expected_x).abs() < 0.01);
    assert!((plan.images[0].frame.width - 16.0).abs() < 0.01);
}

#[test]
fn screen_space_ui_plan_places_vertical_rl_inline_image_at_main_axis_offset() {
    let markup = "甲<img src=\"res://icons/star.png\" width=\"18\" height=\"24\">乙";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Glyph,
        text_writing_mode: UiTextWritingMode::VerticalRl,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::HtmlSubsetV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 48.0, 35.0);
    let layout = layout_text(markup, &style, frame, None);
    let line = &layout.lines[0];
    let inline_run = line
        .runs
        .iter()
        .find(|run| run.text == "\u{fffc}")
        .expect("resolved vertical inline run");
    let inline_visual_index = line.text[..inline_run.visual_range.start]
        .graphemes(true)
        .count();
    let expected_y = line.frame.y
        + line
            .glyph_advances
            .iter()
            .take(inline_visual_index)
            .sum::<f32>();
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-vertical"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(11),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(100, 100),
    );

    assert_eq!(plan.images.len(), 1);
    assert!((plan.images[0].frame.y - expected_y).abs() < 0.01);
    assert!((plan.images[0].frame.width - 18.0).abs() < 0.01);
    assert!((plan.images[0].frame.height - 24.0).abs() < 0.01);
}

#[test]
fn screen_space_ui_plan_does_not_paint_an_inline_widget_placeholder() {
    let markup = "[widget=42|24x16]";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 16.0,
        line_height: 20.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCodeV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 120.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-widget"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(12),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(160, 100),
    );

    assert!(
        plan.vertices.is_empty(),
        "the text renderer must not impersonate a real UI child with a solid quad"
    );
    assert!(plan.images.is_empty());
    assert!(plan.native_texts.is_empty());
}

#[cfg(feature = "profiling")]
#[test]
fn rich_inline_geometry_profile_reports_existing_prefix_reconstruction_work() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "rich-inline-geometry-work".to_owned();
    config.max_spans = 32;
    config.max_counters = 64;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let markup = concat!(
        "a<img src=\"res://icons/one.png\" width=\"12\" height=\"14\">",
        "b<img src=\"res://icons/two.png\" width=\"12\" height=\"14\">",
        "c<img src=\"res://icons/three.png\" width=\"12\" height=\"14\">d",
    );
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 12.0,
        line_height: 16.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::HtmlSubsetV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 320.0, 24.0);
    let layout = layout_text(markup, &style, frame, None);
    let mut expected_inline_runs = 0_usize;
    let mut expected_line_run_probes = 0_usize;
    let mut expected_prefix_graphemes = 0_usize;
    for line in &layout.lines {
        for (run_index, run) in line.runs.iter().enumerate() {
            if run.text != "\u{fffc}" {
                continue;
            }
            expected_inline_runs = expected_inline_runs.saturating_add(1);
            expected_line_run_probes = expected_line_run_probes.saturating_add(run_index + 1);
            expected_prefix_graphemes = expected_prefix_graphemes
                .saturating_add(line.text[..run.visual_range.start].graphemes(true).count());
        }
    }
    assert_eq!(expected_inline_runs, 3);

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-profile"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(613),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(360, 80),
    );
    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(!crate::core::diagnostics::profiling::reset_capture().active);
    let counter = |name: &str| {
        snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value as usize)
            .unwrap_or_else(|| panic!("missing rich inline profile counter {name}"))
    };

    assert_eq!(plan.images.len(), expected_inline_runs);
    assert_eq!(counter("rich_inline_run_count"), expected_inline_runs);
    assert_eq!(
        counter("rich_inline_line_probe_count"),
        expected_inline_runs,
        "every inline object currently restarts the one-line search"
    );
    assert_eq!(
        counter("rich_inline_line_run_probe_count"),
        expected_line_run_probes
    );
    assert_eq!(
        counter("rich_inline_prefix_grapheme_count"),
        expected_prefix_graphemes
    );
    assert_eq!(
        counter("rich_inline_prefix_advance_count"),
        expected_prefix_graphemes
    );
    assert_eq!(
        counter("rich_inline_paint_frame_match_count"),
        expected_inline_runs
    );
    assert_eq!(counter("rich_inline_paint_frame_mismatch_count"), 0);
}

#[cfg(all(feature = "profiling", windows))]
#[path = "rich_inline_profile.rs"]
mod rich_inline_profile;
