use super::*;
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::text::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, register_resolved_text_glyph_artifact,
};
use std::sync::Arc;

#[test]
fn screen_space_ui_plan_requires_glyph_artifact_for_plain_resolved_layout() {
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
                                frame: UiFrame::new(20.0, 20.0, 50.0, 12.0),
                                source_range: UiTextRange { start: 0, end: 10 },
                                visual_range: UiTextRange { start: 0, end: 10 },
                                measured_width: 50.0,
                                glyph_advances: vec![],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha Beta".to_string(),
                                    source_range: UiTextRange { start: 0, end: 10 },
                                    visual_range: UiTextRange { start: 0, end: 10 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                            UiResolvedTextLine {
                                text: "Gamma".to_string(),
                                frame: UiFrame::new(35.0, 32.0, 25.0, 12.0),
                                source_range: UiTextRange { start: 11, end: 16 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                measured_width: 25.0,
                                glyph_advances: vec![],
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
        },
        UVec2::new(160, 120),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_does_not_shape_visual_bidi_runs_without_an_artifact() {
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
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some("abc אבג".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(160, 120),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_preserves_plain_glyph_artifact_through_sdf_routing() {
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
    });
    let mut plan = plan_screen_space_ui_batches(
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
        },
        UVec2::new(120, 80),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert!(plan.sdf_texts[0].preserve_shaped_glyphs);
    assert!(plan.sdf_texts[0].shaped_glyphs.is_empty());
    let artifact_line = plan.sdf_texts[0]
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

    let batch = &mut plan.sdf_texts[0];
    batch
        .glyph_artifact_line
        .as_mut()
        .expect("glyph artifact line")
        .font_generation = u64::MAX;
    super::super::text_advances::refresh_screen_space_text_batch_glyphs(batch);

    let refreshed_line = batch
        .glyph_artifact_line
        .as_ref()
        .expect("glyph artifact line")
        .refreshed_line
        .as_ref()
        .expect("font generation rebuild must retain a text-owned line");
    assert!(batch.shaped_glyphs.is_empty());
    assert!(
        refreshed_line
            .glyphs
            .windows(2)
            .all(|pair| pair[0].source_range.start >= pair[1].source_range.start)
    );
    assert!(
        refreshed_line
            .glyphs
            .iter()
            .all(|glyph| glyph.source_range.start < glyph.source_range.end
                && glyph.source_range.end <= 8)
    );
    assert_eq!(
        refreshed_line
            .glyphs
            .iter()
            .map(|glyph| glyph.advance)
            .sum::<f32>(),
        40.0
    );
}
