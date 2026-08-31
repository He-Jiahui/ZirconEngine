use super::*;
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::text::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, register_resolved_text_glyph_artifact,
};
use crate::ui::surface::layout_text;
use std::sync::Arc;

#[test]
fn screen_space_ui_plan_renders_source_isomorphic_plain_layout_without_glyph_artifact() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(4),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 90.0, 48.0),
                    clip_frame: Some(UiFrame::new(0.0, 0.0, 120.0, 48.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 50.0,
                        measured_height: 24.0,
                        source_range: UiTextRange { start: 0, end: 16 },
                        lines: vec![
                            UiResolvedTextLine {
                                text: "Alpha Beta".to_string(),
                                placement_frame: UiFrame::default(),
                                frame: UiFrame::new(20.0, 20.0, 50.0, 12.0),
                                source_range: UiTextRange { start: 0, end: 10 },
                                visual_range: UiTextRange { start: 0, end: 10 },
                                measured_width: 50.0,
                                glyph_advances: vec![5.0; 10],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: Vec::new(),
                                ellipsized: false,
                            },
                            UiResolvedTextLine {
                                text: "Gamma".to_string(),
                                placement_frame: UiFrame::default(),
                                frame: UiFrame::new(35.0, 32.0, 25.0, 12.0),
                                source_range: UiTextRange { start: 11, end: 16 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                measured_width: 25.0,
                                glyph_advances: vec![5.0; 5],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Gamma".to_string(),
                                    source_range: UiTextRange { start: 11, end: 16 },
                                    visual_range: UiTextRange { start: 0, end: 5 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                        ],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some("Alpha Beta Gamma".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(160, 120),
    );

    assert_eq!(plan.native_texts.len(), 2);
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(plan.native_texts[0].text, "Alpha Beta");
    assert_eq!(plan.native_texts[0].glyph_advances, vec![5.0; 10]);
    assert_eq!(
        plan.native_texts[0].frame,
        UiFrame::new(20.0, 20.0, 50.0, 12.0)
    );
    assert_eq!(plan.native_texts[1].text, "Gamma");
    assert_eq!(plan.native_texts[1].glyph_advances, vec![5.0; 5]);
    assert_eq!(
        plan.native_texts[1].frame,
        UiFrame::new(35.0, 32.0, 25.0, 12.0)
    );
    assert!(
        plan.native_texts
            .iter()
            .all(|text| text.is_source_isomorphic_layout_line)
    );
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            source_isomorphic_fallback_command_count: 1,
            missing_artifact_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn screen_space_ui_plan_keeps_artifact_backed_cjk_layout_in_native_path() {
    let frame = UiFrame::new(24.0, 26.0, 192.0, 154.0);
    let style = UiResolvedStyle {
        foreground_color: Some("#e0f4ff".to_string()),
        font_family: Some("Zircon Noto Sans CJK SC Proof".to_string()),
        language: Some("zh-Hans".to_string()),
        font_size: 24.0,
        line_height: 30.0,
        text_align: UiTextAlign::Left,
        wrap: UiTextWrap::Word,
        text_render_mode: UiTextRenderMode::Native,
        ..UiResolvedStyle::default()
    };
    let text = "中文排版引擎文本与布局 中文排版引擎文本与布局";
    let layout = layout_text(text, &style, frame, Some(frame));

    assert!(
        layout.lines.len() > 1,
        "CJK input must use the layout owner"
    );
    assert!(
        layout.rich_text_artifact.is_some(),
        "plain resolved layouts must retain their canonical glyph artifact"
    );
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.native-layout-artifact"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(704),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: Some(frame),
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(text.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(360, 220),
    );

    assert!(plan.sdf_texts.is_empty());
    assert!(
        !plan.native_texts.is_empty(),
        "horizontal artifact-backed layouts must enter the native atlas path"
    );
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| batch.glyph_artifact_line.is_some()),
        "native layout batches must preserve the Text03 artifact rather than re-shaping"
    );
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            artifact_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn screen_space_ui_plan_rejects_visual_bidi_without_artifact_and_suppresses_decorations() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(19),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 70.0, 12.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
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
                        measured_width: 70.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 10 },
                        lines: vec![UiResolvedTextLine {
                            text: "abc גבא".to_string(),
                            placement_frame: UiFrame::default(),
                            frame: UiFrame::new(10.0, 20.0, 70.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 10 },
                            visual_range: UiTextRange { start: 0, end: 10 },
                            measured_width: 70.0,
                            glyph_advances: vec![10.0; 7],
                            baseline: 9.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "abc ".to_string(),
                                    source_range: UiTextRange { start: 0, end: 4 },
                                    visual_range: UiTextRange { start: 0, end: 4 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "ג".to_string(),
                                    source_range: UiTextRange { start: 8, end: 10 },
                                    visual_range: UiTextRange { start: 4, end: 6 },
                                    direction: UiTextDirection::RightToLeft,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "ב".to_string(),
                                    source_range: UiTextRange { start: 6, end: 8 },
                                    visual_range: UiTextRange { start: 6, end: 8 },
                                    direction: UiTextDirection::RightToLeft,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "א".to_string(),
                                    source_range: UiTextRange { start: 4, end: 6 },
                                    visual_range: UiTextRange { start: 8, end: 10 },
                                    direction: UiTextDirection::RightToLeft,
                                },
                            ],
                            ellipsized: false,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: Some(UiEditableTextState {
                            text: "abc אבג".to_string(),
                            caret: UiTextCaret {
                                offset: 3,
                                affinity: UiTextCaretAffinity::Downstream,
                            },
                            selection: Some(UiTextSelection {
                                anchor: 0,
                                focus: 3,
                            }),
                            composition: None,
                            read_only: false,
                        }),
                        rich_text_artifact: None,
                    }),
                    text: Some("abc אבג".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(160, 120),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.vertices.is_empty());
    assert!(plan.draws.is_empty());
    assert!(plan.post_text_draws.is_empty());
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            missing_artifact_count: 1,
            rejected_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn screen_space_ui_plan_rejects_non_finite_plain_layout_geometry_before_source_fallback() {
    let text = "Alpha";
    let frame = UiFrame::new(10.0, 20.0, 50.0, 12.0);
    let source_range = UiTextRange {
        start: 0,
        end: text.len(),
    };
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.invalid-plain-geometry"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(21),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        font_size: 10.0,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 50.0,
                        measured_height: 12.0,
                        source_range,
                        lines: vec![UiResolvedTextLine {
                            text: text.to_string(),
                            placement_frame: UiFrame::default(),
                            frame,
                            source_range,
                            visual_range: source_range,
                            measured_width: 50.0,
                            glyph_advances: vec![10.0, f32::NAN, 10.0, 10.0, 10.0],
                            baseline: 9.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: Vec::new(),
                            ellipsized: false,
                        }],
                        editable: Some(UiEditableTextState {
                            text: text.to_string(),
                            caret: UiTextCaret {
                                offset: text.len(),
                                affinity: UiTextCaretAffinity::Downstream,
                            },
                            selection: Some(UiTextSelection {
                                anchor: 0,
                                focus: text.len(),
                            }),
                            composition: None,
                            read_only: false,
                        }),
                        ..UiResolvedTextLayout::default()
                    }),
                    text: Some(text.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(120, 80),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.vertices.is_empty());
    assert!(plan.draws.is_empty());
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
fn screen_space_ui_plan_does_not_re_shape_nonempty_text_with_safe_empty_layout() {
    let text = "Safe failure";
    let frame = UiFrame::new(10.0, 20.0, 90.0, 14.0);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.safe-empty-layout"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(22),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        font_size: 12.0,
                        line_height: 14.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        font_size: 12.0,
                        line_height: 14.0,
                        measured_width: 0.0,
                        measured_height: 14.0,
                        source_range: UiTextRange {
                            start: 0,
                            end: text.len(),
                        },
                        lines: Vec::new(),
                        overflow_clipped: true,
                        ..UiResolvedTextLayout::default()
                    }),
                    text: Some(text.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(140, 80),
    );

    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
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
fn screen_space_ui_plan_rejects_non_finite_command_frame_before_fallback() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.invalid-command-frame"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(23),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(f32::NAN, 20.0, 90.0, 14.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        font_size: 12.0,
                        line_height: 14.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Invalid frame".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(160, 64),
    );

    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(
        plan.resolved_glyph_artifact_routes.rejected_command_count,
        1
    );
    assert_eq!(
        plan.resolved_glyph_artifact_routes
            .incomplete_artifact_count,
        1
    );
}

#[test]
fn screen_space_ui_plan_reports_a_valid_visual_only_layout_route() {
    let source = "hidden";
    let source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.visual-only"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(20),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 24.0, 12.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        font_size: 10.0,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 24.0,
                        measured_height: 12.0,
                        source_range,
                        lines: vec![UiResolvedTextLine {
                            text: "…".to_string(),
                            placement_frame: UiFrame::default(),
                            frame: UiFrame::new(10.0, 20.0, 24.0, 12.0),
                            source_range,
                            visual_range: UiTextRange { start: 0, end: 3 },
                            measured_width: 24.0,
                            glyph_advances: vec![24.0],
                            baseline: 9.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: Vec::new(),
                            ellipsized: true,
                        }],
                        ..UiResolvedTextLayout::default()
                    }),
                    text: Some(source.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(120, 80),
    );

    assert_eq!(plan.sdf_texts.len(), 1);
    assert_eq!(plan.sdf_texts[0].text, "…");
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            visual_only_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn screen_space_ui_plan_preserves_plain_glyph_artifact_through_native_routing() {
    let glyph = |glyph_id, source_range| TextGlyph {
        glyph_id,
        source_range,
        visual_range: 0..0,
        advance: 10.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 1,
        flags: TextGlyphFlags {
            right_to_left: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: true,
    };
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("سلام"),
        source_text_origin: 0,
        font_generation: 0,
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![
                glyph(104, 6..8),
                glyph(103, 4..6),
                glyph(102, 2..4),
                glyph(101, 0..2),
            ],
            layout_line: UiResolvedTextLine {
                text: "مالس".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(10.0, 20.0, 40.0, 12.0),
                source_range: UiTextRange { start: 0, end: 8 },
                visual_range: UiTextRange { start: 0, end: 8 },
                measured_width: 40.0,
                glyph_advances: vec![10.0; 4],
                baseline: 9.0,
                direction: UiTextDirection::RightToLeft,
                runs: Vec::new(),
                ellipsized: false,
            },
        })],
        logical_virtual_line_sequences: None,
    });
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.glyph-artifact"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(23),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 40.0, 12.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::RightToLeft,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 40.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 8 },
                        lines: vec![UiResolvedTextLine {
                            text: "مالس".to_string(),
                            placement_frame: UiFrame::default(),
                            frame: UiFrame::new(10.0, 20.0, 40.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 8 },
                            visual_range: UiTextRange { start: 0, end: 8 },
                            measured_width: 40.0,
                            glyph_advances: vec![10.0; 4],
                            baseline: 9.0,
                            direction: UiTextDirection::RightToLeft,
                            runs: Vec::new(),
                            ellipsized: false,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: Some(register_resolved_text_glyph_artifact(
                            Arc::clone(&artifact),
                        )),
                    }),
                    text: Some("سلام".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(120, 80),
    );

    assert!(plan.sdf_texts.is_empty());
    assert_eq!(plan.native_texts.len(), 1);
    assert!(plan.native_texts[0].preserve_shaped_glyphs);
    assert!(plan.native_texts[0].shaped_glyphs.is_empty());
    let artifact_line = plan.native_texts[0]
        .glyph_artifact_line
        .as_ref()
        .expect("text-owned glyph artifact line");
    assert!(Arc::ptr_eq(&artifact_line.artifact, &artifact));
    assert_eq!(
        artifact_line
            .glyphs()
            .expect("glyph artifact source line")
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![104, 103, 102, 101]
    );
    assert_eq!(
        artifact_line
            .glyphs()
            .expect("glyph artifact source line")
            .iter()
            .map(|glyph| glyph.advance)
            .collect::<Vec<_>>(),
        vec![10.0; 4]
    );
}
