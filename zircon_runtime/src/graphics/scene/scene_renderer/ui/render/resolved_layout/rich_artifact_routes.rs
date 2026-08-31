use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiResolvedTextLayout, UiTextPaintRun, UiTextRange};

use crate::text::{
    resolve_resolved_text_glyph_artifact, resolve_rich_text_glyph_run_artifact_at,
    resolved_text_glyph_artifact_line_matches_layout, resolved_text_line_requires_visual_fallback,
};

use super::super::ScreenSpaceUiGlyphArtifactLine;
use super::ResolvedGlyphArtifactRejection;

pub(super) struct RichTextGlyphArtifactRun {
    pub(super) glyph_artifact_line: ScreenSpaceUiGlyphArtifactLine,
    pub(super) style_source_range: Option<UiTextRange>,
}

pub(super) enum RichTextGlyphArtifactRoute {
    Artifact(RichTextGlyphArtifactRun),
    VisualOnly,
    Rejected(ResolvedGlyphArtifactRejection),
}

pub(super) enum RichTextGlyphArtifactRouteBatch {
    Complete(Vec<RichTextGlyphArtifactRoute>),
    PaintLayoutMismatch,
}

pub(super) fn rich_text_glyph_artifact_runs(
    layout: &UiResolvedTextLayout,
    paint_runs: &[UiTextPaintRun],
) -> RichTextGlyphArtifactRouteBatch {
    let reject_all = |reason| {
        if !paint_runs_match_layout(layout, paint_runs) {
            return RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch;
        }
        RichTextGlyphArtifactRouteBatch::Complete(
            (0..paint_runs.len())
                .map(|_| RichTextGlyphArtifactRoute::Rejected(reason))
                .collect::<Vec<_>>(),
        )
    };
    let Some(handle) = layout.rich_text_artifact.as_ref() else {
        return reject_all(ResolvedGlyphArtifactRejection::Missing);
    };
    let Some(artifact) = resolve_resolved_text_glyph_artifact(handle) else {
        return reject_all(ResolvedGlyphArtifactRejection::Missing);
    };
    if artifact.lines.len() != layout.lines.len() {
        return reject_all(ResolvedGlyphArtifactRejection::Incomplete);
    }

    let mut routes = Vec::with_capacity(paint_runs.len());
    let mut paint_index = 0_usize;
    let mut directory_index = 0_usize;
    for (line_index, line) in layout.lines.iter().enumerate() {
        let artifact_line = artifact.lines.get(line_index).and_then(Option::as_ref);
        let line_rejection = artifact_line.and_then(|_| {
            (!resolved_text_glyph_artifact_line_matches_layout(artifact.as_ref(), line_index, line))
                .then_some(ResolvedGlyphArtifactRejection::Stale)
        });
        let visual_only =
            artifact_line.is_none() && resolved_text_line_requires_visual_fallback(line);

        for run in &line.runs {
            let run_directory_index = artifact_line.map(|_| {
                let current = directory_index;
                directory_index = directory_index.saturating_add(1);
                current
            });
            if run.text.is_empty() {
                continue;
            }
            let Some(paint_run) = paint_runs.get(paint_index) else {
                return RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch;
            };
            if paint_run.text != run.text
                || paint_run.source_range != run.source_range
                || paint_run.visual_range != run.visual_range
            {
                return RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch;
            }
            paint_index = paint_index.saturating_add(1);

            let route = if let Some(rejection) = line_rejection {
                RichTextGlyphArtifactRoute::Rejected(rejection)
            } else if visual_only {
                RichTextGlyphArtifactRoute::VisualOnly
            } else if let (Some(artifact_line), Some(run_directory_index)) =
                (artifact_line, run_directory_index)
            {
                match resolve_rich_text_glyph_run_artifact_at(
                    handle,
                    run_directory_index,
                    line_index,
                    run.source_range,
                    run.visual_range,
                ) {
                    Some(resolved)
                        if Arc::ptr_eq(&resolved.artifact, &artifact)
                            && resolved.glyph_range.start <= resolved.glyph_range.end
                            && resolved.glyph_range.end <= artifact_line.glyphs.len() =>
                    {
                        RichTextGlyphArtifactRoute::Artifact(RichTextGlyphArtifactRun {
                            glyph_artifact_line: ScreenSpaceUiGlyphArtifactLine {
                                artifact: resolved.artifact,
                                line_index,
                                font_generation: artifact.font_generation,
                                glyph_range: resolved.glyph_range,
                            },
                            style_source_range: resolved.style_source_range,
                        })
                    }
                    _ => RichTextGlyphArtifactRoute::Rejected(
                        ResolvedGlyphArtifactRejection::Incomplete,
                    ),
                }
            } else {
                RichTextGlyphArtifactRoute::Rejected(ResolvedGlyphArtifactRejection::Incomplete)
            };
            routes.push(route);
        }
    }
    if paint_index != paint_runs.len() {
        return RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch;
    }
    RichTextGlyphArtifactRouteBatch::Complete(routes)
}

fn paint_runs_match_layout(layout: &UiResolvedTextLayout, paint_runs: &[UiTextPaintRun]) -> bool {
    let mut paint_index = 0_usize;
    for run in layout.lines.iter().flat_map(|line| &line.runs) {
        if run.text.is_empty() {
            continue;
        }
        let Some(paint_run) = paint_runs.get(paint_index) else {
            return false;
        };
        if paint_run.text != run.text
            || paint_run.source_range != run.source_range
            || paint_run.visual_range != run.visual_range
        {
            return false;
        }
        paint_index = paint_index.saturating_add(1);
    }
    paint_index == paint_runs.len()
}
