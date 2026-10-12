use std::sync::Arc;

use super::*;
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::text::{
    register_resolved_text_glyph_artifact, ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine,
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLine, UiResolvedTextRun, UiTextAlign, UiTextOverflow,
    UiTextRunKind, UiTextWrap, UiTextWritingMode,
};

#[test]
fn ellipsized_line_keeps_its_synthetic_visual_text() {
    let layout = UiResolvedTextLayout {
        text_align: UiTextAlign::Left,
        wrap: UiTextWrap::None,
        direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        overflow: UiTextOverflow::Ellipsis,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 30.0,
        measured_height: 14.0,
        source_range: UiTextRange { start: 0, end: 6 },
        lines: vec![UiResolvedTextLine {
            text: "ab…".to_string(),
            placement_frame: UiFrame::default(),
            frame: UiFrame::new(4.0, 8.0, 30.0, 14.0),
            source_range: UiTextRange { start: 0, end: 6 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 30.0,
            glyph_advances: vec![10.0; 3],
            baseline: 10.0,
            direction: UiTextDirection::LeftToRight,
            runs: Vec::new(),
            ellipsized: true,
        }],
        boxes: Vec::new(),
        overflow_clipped: true,
        editable: None,
        rich_text_artifact: None,
    };

    let batches = logical_text_batches(&layout).expect("ellipsis uses its visual fallback");

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].text, "ab…");
    assert_eq!(batches[0].source_range, UiTextRange { start: 0, end: 6 });
}

#[test]
fn virtual_source_run_keeps_its_synthetic_visual_text() {
    let layout = UiResolvedTextLayout {
        text_align: UiTextAlign::Justify,
        wrap: UiTextWrap::None,
        direction: UiTextDirection::RightToLeft,
        writing_mode: UiTextWritingMode::HorizontalTb,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 12.0,
        measured_height: 14.0,
        source_range: UiTextRange { start: 0, end: 2 },
        lines: vec![UiResolvedTextLine {
            text: "ـ".to_string(),
            placement_frame: UiFrame::default(),
            frame: UiFrame::new(4.0, 8.0, 12.0, 14.0),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            measured_width: 12.0,
            glyph_advances: vec![12.0],
            baseline: 10.0,
            direction: UiTextDirection::RightToLeft,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "ـ".to_string(),
                source_range: UiTextRange { start: 2, end: 2 },
                visual_range: UiTextRange { start: 0, end: 2 },
                direction: UiTextDirection::RightToLeft,
            }],
            ellipsized: false,
        }],
        boxes: Vec::new(),
        overflow_clipped: false,
        editable: None,
        rich_text_artifact: None,
    };

    let batches = logical_text_batches(&layout).expect("virtual text uses visual fallback");

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].text, "ـ");
    assert!(batches[0].glyph_artifact_line.is_none());
}

#[test]
fn glyph_artifact_batches_keep_full_line_glyphs_without_run_local_reshaping() {
    let mut layout = UiResolvedTextLayout {
        text_align: UiTextAlign::Left,
        wrap: UiTextWrap::None,
        direction: UiTextDirection::RightToLeft,
        writing_mode: UiTextWritingMode::HorizontalTb,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 40.0,
        measured_height: 14.0,
        source_range: UiTextRange { start: 0, end: 8 },
        lines: vec![UiResolvedTextLine {
            text: "مالس".to_string(),
            placement_frame: UiFrame::default(),
            frame: UiFrame::new(0.0, 0.0, 40.0, 14.0),
            source_range: UiTextRange { start: 0, end: 8 },
            visual_range: UiTextRange { start: 0, end: 8 },
            measured_width: 40.0,
            glyph_advances: vec![10.0; 4],
            baseline: 10.0,
            direction: UiTextDirection::RightToLeft,
            runs: Vec::new(),
            ellipsized: false,
        }],
        boxes: Vec::new(),
        overflow_clipped: false,
        editable: None,
        rich_text_artifact: None,
    };
    assert!(matches!(
        logical_text_batches(&layout),
        Err(ResolvedGlyphArtifactRejection::Missing)
    ));
    layout.rich_text_artifact = Some(register_resolved_text_glyph_artifact(Arc::new(
        ResolvedTextGlyphArtifact {
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
                layout_line: layout.lines[0].clone(),
            })],
            logical_virtual_line_sequences: None,
        },
    )));

    let batches = logical_text_batches(&layout).expect("glyph artifact batches");

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].text, "مالس");
    assert_eq!(batches[0].glyph_advances, vec![10.0; 4]);
    assert_eq!(
        batches[0]
            .glyph_artifact_line
            .as_ref()
            .expect("glyph artifact must bypass visual run shaping")
            .glyphs()
            .expect("text-owned glyph artifact line")
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![104, 103, 102, 101]
    );
}

#[test]
fn glyph_artifact_batches_report_stale_and_incomplete_layout_ownership() {
    let artifact_line = UiResolvedTextLine {
        text: "مالس".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 40.0, 14.0),
        source_range: UiTextRange { start: 0, end: 8 },
        visual_range: UiTextRange { start: 0, end: 8 },
        measured_width: 40.0,
        glyph_advances: vec![10.0; 4],
        baseline: 10.0,
        direction: UiTextDirection::RightToLeft,
        runs: Vec::new(),
        ellipsized: false,
    };
    let mut layout = UiResolvedTextLayout {
        text_align: UiTextAlign::Left,
        wrap: UiTextWrap::None,
        direction: UiTextDirection::RightToLeft,
        writing_mode: UiTextWritingMode::HorizontalTb,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 40.0,
        measured_height: 14.0,
        source_range: UiTextRange { start: 0, end: 8 },
        lines: vec![artifact_line.clone()],
        boxes: Vec::new(),
        overflow_clipped: false,
        editable: None,
        rich_text_artifact: Some(register_resolved_text_glyph_artifact(Arc::new(
            ResolvedTextGlyphArtifact {
                source_text: Arc::from("سلام"),
                source_text_origin: 0,
                font_generation: 0,
                font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
                style: UiResolvedStyle::default(),
                writing_mode: UiTextWritingMode::HorizontalTb,
                lines: vec![Some(ResolvedTextGlyphArtifactLine {
                    glyphs: vec![glyph(104, 6..8), glyph(103, 4..6)],
                    layout_line: artifact_line.clone(),
                })],
                logical_virtual_line_sequences: None,
            },
        ))),
    };
    layout.lines[0].glyph_advances[0] += 2.0;

    assert!(matches!(
        logical_text_batches(&layout),
        Err(ResolvedGlyphArtifactRejection::Stale)
    ));

    layout.lines[0] = artifact_line;
    layout.rich_text_artifact = Some(register_resolved_text_glyph_artifact(Arc::new(
        ResolvedTextGlyphArtifact {
            source_text: Arc::from("سلام"),
            source_text_origin: 0,
            font_generation: 0,
            font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
            style: UiResolvedStyle::default(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            lines: vec![None],
            logical_virtual_line_sequences: None,
        },
    )));
    assert!(matches!(
        logical_text_batches(&layout),
        Err(ResolvedGlyphArtifactRejection::Incomplete)
    ));
}

#[test]
fn artifact_route_report_keeps_rejection_reasons_distinct() {
    let mut report = ScreenSpaceUiResolvedGlyphArtifactRouteReport::default();

    report.record(ResolvedGlyphArtifactRouteReceipt::SourceIsomorphicFallback(
        ResolvedGlyphArtifactRejection::Stale,
    ));
    report.record(ResolvedGlyphArtifactRouteReceipt::Rejected(
        ResolvedGlyphArtifactRejection::Incomplete,
    ));

    assert_eq!(
        report,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            source_isomorphic_fallback_command_count: 1,
            stale_artifact_count: 1,
            incomplete_artifact_count: 1,
            rejected_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn glyph_artifact_batches_keep_the_text_owner_without_graphics_projection() {
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("א"),
        source_text_origin: 4,
        font_generation: 0,
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![glyph(11, 4..6)],
            layout_line: UiResolvedTextLine {
                text: "א".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(0.0, 0.0, 10.0, 14.0),
                source_range: UiTextRange { start: 4, end: 6 },
                visual_range: UiTextRange { start: 4, end: 6 },
                measured_width: 10.0,
                glyph_advances: vec![10.0],
                baseline: 10.0,
                direction: UiTextDirection::LeftToRight,
                runs: Vec::new(),
                ellipsized: false,
            },
        })],
        logical_virtual_line_sequences: None,
    });
    let layout = UiResolvedTextLayout {
        text_align: UiTextAlign::Left,
        wrap: UiTextWrap::None,
        direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 10.0,
        measured_height: 14.0,
        source_range: UiTextRange { start: 4, end: 6 },
        lines: vec![artifact.lines[0]
            .as_ref()
            .expect("artifact line")
            .layout_line
            .clone()],
        boxes: Vec::new(),
        overflow_clipped: false,
        editable: None,
        rich_text_artifact: Some(register_resolved_text_glyph_artifact(Arc::clone(&artifact))),
    };

    let batches = logical_text_batches(&layout).expect("artifact layout batches");
    let artifact_line = batches[0]
        .glyph_artifact_line
        .as_ref()
        .expect("text-owned artifact line");

    assert!(Arc::ptr_eq(&artifact_line.artifact, &artifact));
    assert_eq!(artifact_line.source_scalar(&glyph(11, 4..6)), 'א');
}

fn glyph(glyph_id: u32, source_range: std::ops::Range<usize>) -> TextGlyph {
    TextGlyph {
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
    }
}
