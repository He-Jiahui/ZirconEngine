use std::sync::Arc;

use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextDirection,
    UiTextRange, UiTextRunPaintStyle, UiTextWrap,
};

use crate::text::{
    resolve_resolved_text_glyph_artifact, resolved_text_line_requires_visual_fallback,
};

use super::text_provenance::has_source_isomorphic_plain_text_provenance;
use super::{
    push_text_batch, PlannedScreenSpaceUi, ScreenSpaceUiBackgroundTracker,
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextRouteContext,
};

pub(super) struct ResolvedLayoutTextBatch {
    pub(super) text: String,
    pub(super) frame: UiFrame,
    pub(super) source_range: UiTextRange,
    pub(super) glyph_advances: Vec<f32>,
    pub(super) direction: UiTextDirection,
    pub(super) glyph_artifact_line: Option<ScreenSpaceUiGlyphArtifactLine>,
    pub(super) is_source_isomorphic: bool,
}

pub(super) fn logical_text_batches(
    layout: &UiResolvedTextLayout,
) -> Option<Vec<ResolvedLayoutTextBatch>> {
    let mut batches = Vec::new();
    if let Some(artifact) = layout
        .rich_text_artifact
        .as_ref()
        .and_then(resolve_resolved_text_glyph_artifact)
    {
        for (index, line) in layout.lines.iter().enumerate() {
            let artifact_line = artifact.lines.get(index).and_then(Option::as_ref);
            if let Some(artifact_line) = artifact_line {
                batches.push(ResolvedLayoutTextBatch {
                    text: line.text.clone(),
                    frame: line.frame,
                    source_range: line.source_range,
                    glyph_advances: line.glyph_advances.clone(),
                    direction: line.direction,
                    glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
                        artifact: Arc::clone(&artifact),
                        line_index: index,
                        refreshed_line: None,
                        font_generation: artifact.font_generation,
                    }),
                    is_source_isomorphic: false,
                });
            } else if resolved_text_line_requires_visual_fallback(line) {
                append_visual_line_batch(&mut batches, line);
            } else {
                return None;
            }
        }
        return Some(batches);
    }
    if layout
        .lines
        .iter()
        .any(|line| !resolved_text_line_requires_visual_fallback(line))
    {
        return None;
    }
    for line in &layout.lines {
        append_visual_line_batch(&mut batches, line);
    }
    Some(batches)
}

pub(super) fn push_resolved_text_layout_line_batches(
    command: &UiRenderCommand,
    route_context: &ScreenSpaceUiTextRouteContext,
    layout: &UiResolvedTextLayout,
    color: [f32; 4],
    viewport: UiFrame,
    raster_scale: f32,
    backgrounds: &ScreenSpaceUiBackgroundTracker,
    plan: &mut PlannedScreenSpaceUi,
) {
    let Some(batches) = logical_text_batches(layout)
        .or_else(|| source_isomorphic_plain_text_batches(command, layout))
    else {
        return;
    };
    for batch in batches {
        push_text_batch(
            command,
            route_context,
            batch.text,
            batch.frame,
            Some(batch.source_range),
            batch.is_source_isomorphic,
            batch.glyph_advances,
            batch.glyph_artifact_line,
            command.style.font.clone(),
            command.style.font_family.clone(),
            command.style.font_weight,
            layout.font_size,
            layout.line_height,
            color,
            UiTextAlign::Left,
            batch.direction,
            layout.writing_mode,
            UiTextWrap::None,
            UiTextRunPaintStyle::default(),
            command.style.text_decorations.clone(),
            viewport,
            raster_scale,
            backgrounds,
            plan,
        );
    }
}

fn source_isomorphic_plain_text_batches(
    command: &UiRenderCommand,
    layout: &UiResolvedTextLayout,
) -> Option<Vec<ResolvedLayoutTextBatch>> {
    layout
        .lines
        .iter()
        .map(|line| {
            has_source_isomorphic_plain_text_provenance(command, line).then(|| {
                ResolvedLayoutTextBatch {
                    text: line.text.clone(),
                    frame: line.frame,
                    source_range: line.source_range,
                    glyph_advances: line.glyph_advances.clone(),
                    direction: line.direction,
                    glyph_artifact_line: None,
                    is_source_isomorphic: true,
                }
            })
        })
        .collect()
}

fn append_visual_line_batch(batches: &mut Vec<ResolvedLayoutTextBatch>, line: &UiResolvedTextLine) {
    batches.push(ResolvedLayoutTextBatch {
        text: line.text.clone(),
        frame: line.frame,
        source_range: line.source_range,
        glyph_advances: line.glyph_advances.clone(),
        direction: line.direction,
        glyph_artifact_line: None,
        is_source_isomorphic: false,
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
    use crate::text::{
        register_resolved_text_glyph_artifact, ResolvedTextGlyphArtifact,
        ResolvedTextGlyphArtifactLine,
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
        assert!(logical_text_batches(&layout).is_none());
        layout.rich_text_artifact = Some(register_resolved_text_glyph_artifact(Arc::new(
            ResolvedTextGlyphArtifact {
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
                    layout_line: layout.lines[0].clone(),
                })],
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
    fn glyph_artifact_batches_keep_the_text_owner_without_graphics_projection() {
        let artifact = Arc::new(ResolvedTextGlyphArtifact {
            source_text: Arc::from("א"),
            source_text_origin: 4,
            font_generation: 0,
            style: UiResolvedStyle::default(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            lines: vec![Some(ResolvedTextGlyphArtifactLine {
                glyphs: vec![glyph(11, 4..6)],
                layout_line: UiResolvedTextLine {
                    text: "א".to_string(),
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
}
