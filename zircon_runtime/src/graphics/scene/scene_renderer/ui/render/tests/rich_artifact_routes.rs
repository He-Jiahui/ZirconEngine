use super::*;

#[test]
fn rich_paint_run_projection_mismatch_fails_closed_without_line_fallback() {
    assert_rich_paint_run_projection_mismatch_fails_closed(false);
}

#[test]
fn rich_paint_run_projection_mismatch_fails_closed_when_glyph_artifact_is_missing() {
    assert_rich_paint_run_projection_mismatch_fails_closed(true);
}

fn assert_rich_paint_run_projection_mismatch_fails_closed(remove_glyph_artifact: bool) {
    let markup = "Alpha **Beta**";
    let style = UiResolvedStyle {
        font_size: 12.0,
        line_height: 16.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 180.0, 24.0);
    let mut layout = layout_text(markup, &style, frame, None);
    assert!(layout.lines[0].runs.len() >= 2);
    layout.lines[0].glyph_advances.clear();
    layout
        .boxes
        .push(zircon_runtime_interface::ui::surface::UiResolvedTextBox {
            range: UiTextRange {
                start: 0,
                end: markup.len(),
            },
            frame,
            background_color: Some(zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(
                0x12, 0x20, 0x2C, 0x80,
            )),
            border_color: Some(zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(
                0x73, 0xD7, 0xFF, 0xFF,
            )),
            border_width: 1.0,
        });
    if remove_glyph_artifact {
        layout.rich_text_artifact = None;
    }

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-paint-run-mismatch"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(615),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#112233".to_string()),
                        ..style
                    },
                    text_layout: Some(layout),
                    text: Some(markup.to_owned()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(220, 64),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.images.is_empty());
    assert_eq!(plan.vertices.len(), 6);
    assert!(plan.post_text_draws.is_empty());
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            incomplete_artifact_count: 1,
            rejected_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn rich_paint_run_non_finite_geometry_rejects_before_partial_materialization() {
    assert_rich_paint_run_mutation_rejects_before_partial_materialization(|runs| {
        runs[1].frame.x = f32::NAN;
    });
}

#[test]
fn rich_paint_run_non_positive_metrics_reject_before_partial_materialization() {
    assert_rich_paint_run_mutation_rejects_before_partial_materialization(|runs| {
        runs[1].font_size = 0.0;
    });
}

fn assert_rich_paint_run_mutation_rejects_before_partial_materialization(
    mutate: impl FnOnce(&mut [zircon_runtime_interface::ui::surface::UiTextPaintRun]),
) {
    let markup = "Alpha **Beta**";
    let style = UiResolvedStyle {
        font_size: 12.0,
        line_height: 16.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 180.0, 24.0);
    let mut layout = layout_text(markup, &style, frame, None);
    layout
        .boxes
        .push(zircon_runtime_interface::ui::surface::UiResolvedTextBox {
            range: UiTextRange {
                start: 0,
                end: markup.len(),
            },
            frame,
            background_color: Some(zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(
                0x12, 0x20, 0x2C, 0x80,
            )),
            border_color: None,
            border_width: 0.0,
        });
    let command = UiRenderCommand {
        node_id: UiNodeId::new(616),
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            background_color: Some("#112233".to_string()),
            ..style
        },
        text_layout: Some(layout),
        text: Some(markup.to_owned()),
        image: None,
        opacity: 1.0,
    };
    let mut paint_elements = command.to_transient_paint_elements(0);
    let text_paint = paint_elements
        .iter_mut()
        .find_map(|element| match &mut element.payload {
            zircon_runtime_interface::ui::surface::UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
        .expect("text paint payload");
    assert!(text_paint.runs.len() >= 2);
    mutate(&mut text_paint.runs);

    let viewport = UiFrame::new(0.0, 0.0, 220.0, 64.0);
    let route_tree_id = Arc::<str>::from("runtime.ui.rich-invalid-run-geometry");
    let backgrounds = ScreenSpaceUiBackgroundTracker::default();
    let mut plan = PlannedScreenSpaceUi::default();
    let rejected = plan_command_batches(
        &command,
        &paint_elements,
        &route_tree_id,
        command.node_id,
        viewport,
        1.0,
        &backgrounds,
        &mut plan,
    );

    assert!(rejected);
    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.images.is_empty());
    assert_eq!(plan.vertices.len(), 6);
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            incomplete_artifact_count: 1,
            rejected_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn rich_multi_run_without_artifact_rejects_command_instead_of_reshaping_runs() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(6),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 150.0, 18.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 150.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 15 },
                        lines: vec![UiResolvedTextLine {
                            text: "Alpha Beta Code".to_string(),
                            placement_frame: UiFrame::default(),
                            frame: UiFrame::new(10.0, 20.0, 150.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 15 },
                            visual_range: UiTextRange { start: 0, end: 15 },
                            measured_width: 150.0,
                            glyph_advances: vec![10.0; 15],
                            baseline: 8.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha ".to_string(),
                                    source_range: UiTextRange { start: 0, end: 6 },
                                    visual_range: UiTextRange { start: 0, end: 6 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Strong,
                                    text: "Beta".to_string(),
                                    source_range: UiTextRange { start: 6, end: 10 },
                                    visual_range: UiTextRange { start: 6, end: 10 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Code,
                                    text: " Code".to_string(),
                                    source_range: UiTextRange { start: 10, end: 15 },
                                    visual_range: UiTextRange { start: 10, end: 15 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                            ],
                            ellipsized: false,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some("Alpha Beta Code".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(220, 80),
    );

    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            missing_artifact_count: 1,
            rejected_command_count: 1,
            rich_rejected_run_count: 3,
            rich_missing_artifact_count: 3,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn rich_unrecoverable_run_rejects_before_source_fallback_partial_materialization() {
    let source = "Alpha hidden";
    let command_frame = UiFrame::new(10.0, 20.0, 180.0, 40.0);
    let first_frame = UiFrame::new(10.0, 20.0, 50.0, 12.0);
    let second_frame = UiFrame::new(10.0, 36.0, 24.0, 12.0);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-command-atomic-rejection"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(617),
                    kind: UiRenderCommandKind::Text,
                    frame: command_frame,
                    clip_frame: Some(command_frame),
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#112233".to_string()),
                        font_size: 12.0,
                        line_height: 16.0,
                        text_render_mode: UiTextRenderMode::Native,
                        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Ellipsis,
                        font_size: 12.0,
                        line_height: 16.0,
                        measured_width: 50.0,
                        measured_height: 28.0,
                        source_range: UiTextRange {
                            start: 0,
                            end: source.len(),
                        },
                        lines: vec![
                            UiResolvedTextLine {
                                text: "Alpha".to_string(),
                                placement_frame: UiFrame::default(),
                                frame: first_frame,
                                source_range: UiTextRange { start: 0, end: 5 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                measured_width: 50.0,
                                glyph_advances: vec![10.0; 5],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha".to_string(),
                                    source_range: UiTextRange { start: 0, end: 5 },
                                    visual_range: UiTextRange { start: 0, end: 5 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                            UiResolvedTextLine {
                                text: "…".to_string(),
                                placement_frame: UiFrame::default(),
                                frame: second_frame,
                                source_range: UiTextRange { start: 6, end: 12 },
                                visual_range: UiTextRange { start: 0, end: 3 },
                                measured_width: 24.0,
                                glyph_advances: vec![24.0],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "…".to_string(),
                                    source_range: UiTextRange { start: 12, end: 12 },
                                    visual_range: UiTextRange { start: 0, end: 3 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: true,
                            },
                        ],
                        boxes: vec![zircon_runtime_interface::ui::surface::UiResolvedTextBox {
                            range: UiTextRange {
                                start: 0,
                                end: source.len(),
                            },
                            frame: command_frame,
                            background_color: Some(
                                zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(
                                    0x12, 0x20, 0x2C, 0x80,
                                ),
                            ),
                            border_color: None,
                            border_width: 0.0,
                        }],
                        overflow_clipped: true,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some(source.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(220, 80),
    );

    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.images.is_empty());
    assert_eq!(plan.vertices.len(), 6);
    assert!(plan.post_text_draws.is_empty());
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            missing_artifact_count: 1,
            rejected_command_count: 1,
            rich_source_isomorphic_fallback_run_count: 1,
            rich_rejected_run_count: 1,
            rich_missing_artifact_count: 2,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn rich_non_isomorphic_run_without_artifact_rejects_renderer_reshape() {
    let source = "hidden";
    let frame = UiFrame::new(10.0, 20.0, 24.0, 18.0);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-missing-non-isomorphic"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(609),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: Some(frame),
                    z_index: 0,
                    style: UiResolvedStyle {
                        font_size: 12.0,
                        line_height: 16.0,
                        text_render_mode: UiTextRenderMode::Native,
                        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Ellipsis,
                        font_size: 12.0,
                        line_height: 16.0,
                        measured_width: 24.0,
                        measured_height: 16.0,
                        source_range: UiTextRange {
                            start: 0,
                            end: source.len(),
                        },
                        lines: vec![UiResolvedTextLine {
                            text: "…".to_string(),
                            placement_frame: UiFrame::default(),
                            frame,
                            source_range: UiTextRange {
                                start: 0,
                                end: source.len(),
                            },
                            visual_range: UiTextRange { start: 0, end: 3 },
                            measured_width: 24.0,
                            glyph_advances: vec![24.0],
                            baseline: 12.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![UiResolvedTextRun {
                                kind: UiTextRunKind::Plain,
                                text: "…".to_string(),
                                source_range: UiTextRange {
                                    start: source.len(),
                                    end: source.len(),
                                },
                                visual_range: UiTextRange { start: 0, end: 3 },
                                direction: UiTextDirection::LeftToRight,
                            }],
                            ellipsized: true,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: true,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some(source.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(100, 80),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            missing_artifact_count: 1,
            rejected_command_count: 1,
            rich_rejected_run_count: 1,
            rich_missing_artifact_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[cfg(feature = "profiling")]
#[test]
fn rich_text_render_reports_canonical_run_artifacts_without_fallback_shape() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "rich-render-canonical-run-artifacts".to_owned();
    config.max_spans = 32;
    config.max_counters = 32;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let markup = "Alpha **Beta** `Code`";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 12.0,
        line_height: 16.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 220.0, 24.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-profile"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(607),
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
        UVec2::new(260, 80),
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(!crate::core::diagnostics::profiling::reset_capture().active);
    let fallback_count = snapshot
        .counters
        .iter()
        .find(|counter| {
            counter.stream == "runtime"
                && counter.name == "rich_render_fallback_shape_request_count"
        })
        .map(|counter| counter.value)
        .expect("rich renderer fallback-shape counter");
    let artifact_count = snapshot
        .counters
        .iter()
        .find(|counter| {
            counter.stream == "runtime" && counter.name == "rich_render_artifact_run_count"
        })
        .map(|counter| counter.value)
        .expect("rich renderer artifact-run counter");
    assert_eq!(artifact_count, plan.native_texts.len() as f64);
    assert!(
        artifact_count >= 3.0,
        "fixture must retain multiple styled runs"
    );
    assert_eq!(fallback_count, 0.0);
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| {
                span.category == "text.render" && span.name == "shape_renderer_fallback"
            })
            .count(),
        0,
        "canonical rich runs must bypass renderer fallback shaping"
    );
    assert_eq!(
        plan.resolved_glyph_artifact_routes.rich_artifact_run_count,
        plan.native_texts.len()
    );
}

#[test]
fn vertical_rich_ellipsis_reports_canonical_artifact_runs() {
    let markup = "[b]abcdef[/b]";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 16.0,
        line_height: 20.0,
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Ellipsis,
        text_writing_mode: UiTextWritingMode::VerticalRl,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCodeV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 24.0, 36.0);
    let layout = layout_text(markup, &style, frame, None);
    assert!(layout.lines[0].ellipsized);

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.vertical-rich-visual-only"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(608),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: Some(frame),
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

    assert!(plan.resolved_glyph_artifact_routes.rich_artifact_run_count > 0);
    assert_eq!(
        plan.resolved_glyph_artifact_routes
            .rich_visual_only_run_count,
        0
    );
    assert_eq!(
        plan.resolved_glyph_artifact_routes.rich_rejected_run_count,
        0
    );
}

#[test]
fn vertical_rich_soft_hyphen_reports_canonical_artifact_runs() {
    let markup = "pre\u{00ad}fix";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 16.0,
        line_height: 20.0,
        wrap: UiTextWrap::Word,
        text_overflow: UiTextOverflow::Clip,
        text_writing_mode: UiTextWritingMode::VerticalRl,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCodeV1,
        ..UiResolvedStyle::default()
    };
    let column_height = crate::ui::surface::measure_text_size("pre-", &style).width + 0.1;
    let frame = UiFrame::new(10.0, 20.0, 80.0, column_height);
    let layout = layout_text(markup, &style, frame, None);
    assert_eq!(layout.lines[0].text, "pre-");

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.vertical-rich-soft-hyphen"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(610),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: Some(frame),
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
        UVec2::new(120, 120),
    );

    assert!(plan.resolved_glyph_artifact_routes.rich_artifact_run_count > 0);
    assert_eq!(
        plan.resolved_glyph_artifact_routes
            .rich_visual_only_run_count,
        0
    );
    assert_eq!(
        plan.resolved_glyph_artifact_routes.rich_rejected_run_count,
        0
    );
}
