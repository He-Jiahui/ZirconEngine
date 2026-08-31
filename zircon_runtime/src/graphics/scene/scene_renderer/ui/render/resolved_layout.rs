use std::sync::Arc;

use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextDirection,
    UiTextRange, UiTextRunPaintStyle, UiTextWrap,
};

use crate::text::{
    resolve_resolved_text_glyph_artifact, resolved_text_glyph_artifact_line_matches_layout,
    resolved_text_line_requires_visual_fallback,
};

use super::text_batches::push_text_batch;
use super::text_provenance::has_source_isomorphic_plain_text_provenance;
use super::{
    PlannedScreenSpaceUi, ScreenSpaceUiBackgroundTracker, ScreenSpaceUiGlyphArtifactLine,
    ScreenSpaceUiTextRouteContext,
};

mod rich_artifact_routes;

pub(super) use rich_artifact_routes::{
    RichTextGlyphArtifactRoute, RichTextGlyphArtifactRouteBatch, rich_text_glyph_artifact_runs,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResolvedGlyphArtifactRejection {
    Missing,
    Stale,
    Incomplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResolvedGlyphArtifactRouteReceipt {
    Artifact,
    VisualOnly,
    SourceIsomorphicFallback(ResolvedGlyphArtifactRejection),
    Rejected(ResolvedGlyphArtifactRejection),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiResolvedGlyphArtifactRouteReport {
    pub(crate) artifact_command_count: usize,
    pub(crate) visual_only_command_count: usize,
    pub(crate) source_isomorphic_fallback_command_count: usize,
    pub(crate) missing_artifact_count: usize,
    pub(crate) stale_artifact_count: usize,
    pub(crate) incomplete_artifact_count: usize,
    pub(crate) rejected_command_count: usize,
    pub(crate) rich_artifact_run_count: usize,
    pub(crate) rich_visual_only_run_count: usize,
    pub(crate) rich_source_isomorphic_fallback_run_count: usize,
    pub(crate) rich_rejected_run_count: usize,
    pub(crate) rich_missing_artifact_count: usize,
    pub(crate) rich_stale_artifact_count: usize,
    pub(crate) rich_incomplete_artifact_count: usize,
}

impl ScreenSpaceUiResolvedGlyphArtifactRouteReport {
    pub(super) fn merge(&mut self, next: Self) {
        self.artifact_command_count = self
            .artifact_command_count
            .saturating_add(next.artifact_command_count);
        self.visual_only_command_count = self
            .visual_only_command_count
            .saturating_add(next.visual_only_command_count);
        self.source_isomorphic_fallback_command_count = self
            .source_isomorphic_fallback_command_count
            .saturating_add(next.source_isomorphic_fallback_command_count);
        self.missing_artifact_count = self
            .missing_artifact_count
            .saturating_add(next.missing_artifact_count);
        self.stale_artifact_count = self
            .stale_artifact_count
            .saturating_add(next.stale_artifact_count);
        self.incomplete_artifact_count = self
            .incomplete_artifact_count
            .saturating_add(next.incomplete_artifact_count);
        self.rejected_command_count = self
            .rejected_command_count
            .saturating_add(next.rejected_command_count);
        self.rich_artifact_run_count = self
            .rich_artifact_run_count
            .saturating_add(next.rich_artifact_run_count);
        self.rich_visual_only_run_count = self
            .rich_visual_only_run_count
            .saturating_add(next.rich_visual_only_run_count);
        self.rich_source_isomorphic_fallback_run_count = self
            .rich_source_isomorphic_fallback_run_count
            .saturating_add(next.rich_source_isomorphic_fallback_run_count);
        self.rich_rejected_run_count = self
            .rich_rejected_run_count
            .saturating_add(next.rich_rejected_run_count);
        self.rich_missing_artifact_count = self
            .rich_missing_artifact_count
            .saturating_add(next.rich_missing_artifact_count);
        self.rich_stale_artifact_count = self
            .rich_stale_artifact_count
            .saturating_add(next.rich_stale_artifact_count);
        self.rich_incomplete_artifact_count = self
            .rich_incomplete_artifact_count
            .saturating_add(next.rich_incomplete_artifact_count);
    }

    pub(super) fn record(&mut self, receipt: ResolvedGlyphArtifactRouteReceipt) {
        match receipt {
            ResolvedGlyphArtifactRouteReceipt::Artifact => {
                self.artifact_command_count = self.artifact_command_count.saturating_add(1);
            }
            ResolvedGlyphArtifactRouteReceipt::VisualOnly => {
                self.visual_only_command_count = self.visual_only_command_count.saturating_add(1);
            }
            ResolvedGlyphArtifactRouteReceipt::SourceIsomorphicFallback(rejection) => {
                self.source_isomorphic_fallback_command_count = self
                    .source_isomorphic_fallback_command_count
                    .saturating_add(1);
                self.record_rejection(rejection);
            }
            ResolvedGlyphArtifactRouteReceipt::Rejected(rejection) => {
                self.rejected_command_count = self.rejected_command_count.saturating_add(1);
                self.record_rejection(rejection);
            }
        }
    }

    pub(super) fn has_activity(self) -> bool {
        self != Self::default()
    }

    pub(super) fn record_rich_run(
        &mut self,
        route: &RichTextGlyphArtifactRoute,
        source_isomorphic_fallback: bool,
    ) {
        match route {
            RichTextGlyphArtifactRoute::Artifact(_) => {
                self.rich_artifact_run_count = self.rich_artifact_run_count.saturating_add(1);
            }
            RichTextGlyphArtifactRoute::VisualOnly => {
                self.rich_visual_only_run_count = self.rich_visual_only_run_count.saturating_add(1);
            }
            RichTextGlyphArtifactRoute::Rejected(rejection) => {
                if source_isomorphic_fallback {
                    self.rich_source_isomorphic_fallback_run_count = self
                        .rich_source_isomorphic_fallback_run_count
                        .saturating_add(1);
                } else {
                    self.rich_rejected_run_count = self.rich_rejected_run_count.saturating_add(1);
                }
                let count = match rejection {
                    ResolvedGlyphArtifactRejection::Missing => {
                        &mut self.rich_missing_artifact_count
                    }
                    ResolvedGlyphArtifactRejection::Stale => &mut self.rich_stale_artifact_count,
                    ResolvedGlyphArtifactRejection::Incomplete => {
                        &mut self.rich_incomplete_artifact_count
                    }
                };
                *count = count.saturating_add(1);
            }
        }
    }

    fn record_rejection(&mut self, rejection: ResolvedGlyphArtifactRejection) {
        let count = match rejection {
            ResolvedGlyphArtifactRejection::Missing => &mut self.missing_artifact_count,
            ResolvedGlyphArtifactRejection::Stale => &mut self.stale_artifact_count,
            ResolvedGlyphArtifactRejection::Incomplete => &mut self.incomplete_artifact_count,
        };
        *count = count.saturating_add(1);
    }
}

pub(super) fn resolved_text_layout_batch_geometry_is_valid(layout: &UiResolvedTextLayout) -> bool {
    layout.font_size.is_finite()
        && layout.font_size > 0.0
        && layout.line_height.is_finite()
        && layout.line_height > 0.0
        && layout
            .lines
            .iter()
            .all(resolved_text_line_batch_geometry_is_valid)
}

fn resolved_text_line_batch_geometry_is_valid(line: &UiResolvedTextLine) -> bool {
    let frame = line.frame;
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && (line.text.is_empty() || frame.width > 0.0 && frame.height > 0.0)
        && line
            .glyph_advances
            .iter()
            .all(|advance| advance.is_finite() && *advance >= 0.0)
}

pub(super) fn logical_text_batches(
    layout: &UiResolvedTextLayout,
) -> Result<Vec<ResolvedLayoutTextBatch>, ResolvedGlyphArtifactRejection> {
    if !resolved_text_layout_batch_geometry_is_valid(layout) {
        return Err(ResolvedGlyphArtifactRejection::Incomplete);
    }
    let mut batches = Vec::new();
    if let Some(artifact) = layout
        .rich_text_artifact
        .as_ref()
        .and_then(resolve_resolved_text_glyph_artifact)
    {
        for (index, line) in layout.lines.iter().enumerate() {
            let artifact_line = artifact.lines.get(index).and_then(Option::as_ref);
            if let Some(artifact_line) = artifact_line {
                if !resolved_text_glyph_artifact_line_matches_layout(artifact.as_ref(), index, line)
                {
                    return Err(ResolvedGlyphArtifactRejection::Stale);
                }
                batches.push(ResolvedLayoutTextBatch {
                    text: line.text.clone(),
                    frame: line.frame,
                    source_range: line.source_range,
                    glyph_advances: line.glyph_advances.clone(),
                    direction: line.direction,
                    glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
                        artifact: Arc::clone(&artifact),
                        line_index: index,
                        font_generation: artifact.font_generation,
                        glyph_range: 0..artifact_line.glyphs.len(),
                    }),
                    is_source_isomorphic: false,
                });
            } else if resolved_text_line_requires_visual_fallback(line) {
                append_visual_line_batch(&mut batches, line);
            } else {
                return Err(ResolvedGlyphArtifactRejection::Incomplete);
            }
        }
        return Ok(batches);
    }
    if layout
        .lines
        .iter()
        .any(|line| !resolved_text_line_requires_visual_fallback(line))
    {
        return Err(ResolvedGlyphArtifactRejection::Missing);
    }
    for line in &layout.lines {
        append_visual_line_batch(&mut batches, line);
    }
    Ok(batches)
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
) -> ResolvedGlyphArtifactRouteReceipt {
    let (batches, receipt) = match logical_text_batches(layout) {
        Ok(batches) => {
            let receipt = if batches
                .iter()
                .any(|batch| batch.glyph_artifact_line.is_some())
            {
                ResolvedGlyphArtifactRouteReceipt::Artifact
            } else {
                ResolvedGlyphArtifactRouteReceipt::VisualOnly
            };
            (batches, receipt)
        }
        Err(rejection) => match source_isomorphic_plain_text_batches(command, layout) {
            Some(batches) => (
                batches,
                ResolvedGlyphArtifactRouteReceipt::SourceIsomorphicFallback(rejection),
            ),
            None => return ResolvedGlyphArtifactRouteReceipt::Rejected(rejection),
        },
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
    receipt
}

fn source_isomorphic_plain_text_batches(
    command: &UiRenderCommand,
    layout: &UiResolvedTextLayout,
) -> Option<Vec<ResolvedLayoutTextBatch>> {
    if !resolved_text_layout_batch_geometry_is_valid(layout) {
        return None;
    }
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
        ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine,
        register_resolved_text_glyph_artifact,
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
            lines: vec![
                artifact.lines[0]
                    .as_ref()
                    .expect("artifact line")
                    .layout_line
                    .clone(),
            ],
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
