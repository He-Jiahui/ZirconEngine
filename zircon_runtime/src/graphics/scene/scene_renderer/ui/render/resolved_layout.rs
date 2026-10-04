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

#[cfg(test)]
#[path = "resolved_layout/tests/logical_batch_capacity_tests.rs"]
mod logical_batch_capacity_tests;

pub(super) use rich_artifact_routes::{
    rich_text_glyph_artifact_runs, RichTextGlyphArtifactRoute, RichTextGlyphArtifactRouteBatch,
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
    let mut batches = Vec::with_capacity(layout.lines.len());
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
#[path = "tests/resolved_layout.rs"]
mod tests;
