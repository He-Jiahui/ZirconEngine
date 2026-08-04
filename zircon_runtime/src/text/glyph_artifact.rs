use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle,
    UiTextCaret, UiTextCaretAffinity, UiTextRange, UiTextWritingMode,
};

use super::font::shared_font_database_generation;
use super::service::project_shaped_glyph_run_for_runtime;
use super::{SharedTextLayoutSession, TextRange, VerticalMode, text_style};
use crate::core::framework::text::{TextGlyph, TextShapeResult};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTextGlyphArtifact {
    pub(crate) source_text: Arc<str>,
    pub(crate) source_text_origin: usize,
    pub(crate) font_generation: u64,
    pub(crate) style: UiResolvedStyle,
    pub(crate) writing_mode: UiTextWritingMode,
    pub(crate) lines: Vec<Option<ResolvedTextGlyphArtifactLine>>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTextGlyphArtifactLine {
    pub(crate) glyphs: Vec<TextGlyph>,
    pub(crate) layout_line: UiResolvedTextLine,
}

/// Returns the visual advance for an interior source offset that a shaped glyph keeps whole.
///
/// The serializable resolved-line DTO carries grapheme advances, while this process-local
/// artifact retains the backend glyph cluster ranges. Returning `None` leaves callers on the DTO
/// path when the layout no longer matches the artifact or the offset is already a legal boundary.
pub(crate) fn resolved_text_glyph_artifact_caret_advance(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
    caret: &UiTextCaret,
) -> Option<f32> {
    let glyphs = matching_artifact_line(artifact, line_index, layout_line)?;
    let backend_cluster_flags = glyphs.iter().any(|glyph| glyph.flags.cluster_start);
    let mut index = 0;
    let mut leading = 0.0;
    let cluster = loop {
        let (cluster, next_index) = glyph_cluster_at(glyphs, index, backend_cluster_flags)?;
        if cluster.source_range.start < caret.offset && caret.offset < cluster.source_range.end {
            break cluster;
        }
        leading += cluster.advance;
        index = next_index;
    };
    let logical_start = matches!(caret.affinity, UiTextCaretAffinity::Upstream);
    Some(if cluster.right_to_left == logical_start {
        leading + cluster.advance
    } else {
        leading
    })
}

/// Resolves a physical visual advance to a legal source caret using backend glyph clusters.
///
/// The serialized DTO has one advance per visual grapheme, which cannot represent a ligature or
/// another backend cluster spanning multiple source offsets. Returning `None` keeps the caller on
/// the DTO source-map path when the text-owned glyph line is unavailable or no longer matches.
pub(crate) fn resolved_text_glyph_artifact_caret_at_advance(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
    visual_advance: f32,
) -> Option<UiTextCaret> {
    let glyphs = matching_artifact_line(artifact, line_index, layout_line)?;
    let backend_cluster_flags = glyphs.iter().any(|glyph| glyph.flags.cluster_start);
    let mut index = 0;
    let mut advance = 0.0;
    let visual_advance = finite_non_negative(visual_advance);
    while index < glyphs.len() {
        let (cluster, next_index) = glyph_cluster_at(glyphs, index, backend_cluster_flags)?;
        if visual_advance <= advance + cluster.advance * 0.5 {
            return Some(cluster_caret(
                cluster.source_range,
                cluster.right_to_left,
                true,
            ));
        }
        advance += cluster.advance;
        index = next_index;
        if index == glyphs.len() {
            return Some(cluster_caret(
                cluster.source_range,
                cluster.right_to_left,
                false,
            ));
        }
    }
    None
}

/// Returns physical advance spans for source ranges that overlap shaped glyph clusters.
pub(crate) fn resolved_text_glyph_artifact_range_advance_spans(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
    range: UiTextRange,
) -> Option<Vec<(f32, f32)>> {
    let glyphs = matching_artifact_line(artifact, line_index, layout_line)?;
    let backend_cluster_flags = glyphs.iter().any(|glyph| glyph.flags.cluster_start);
    let mut spans = Vec::new();
    let mut span_start = None;
    let mut advance = 0.0;
    let mut index = 0;
    while index < glyphs.len() {
        let (cluster, next_index) = glyph_cluster_at(glyphs, index, backend_cluster_flags)?;
        if source_ranges_overlap(cluster.source_range, range) {
            span_start.get_or_insert(advance);
        } else if let Some(start) = span_start.take() {
            spans.push((start, advance));
        }
        advance += cluster.advance;
        index = next_index;
    }
    if let Some(start) = span_start {
        spans.push((start, advance));
    }
    (!spans.is_empty()).then_some(spans)
}

fn matching_artifact_line<'a>(
    artifact: &'a ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
) -> Option<&'a [TextGlyph]> {
    let line = artifact.lines.get(line_index)?.as_ref()?;
    (artifact.font_generation == shared_font_database_generation()
        && line.layout_line == *layout_line)
        .then_some(line.glyphs.as_slice())
}

#[derive(Clone, Copy)]
struct GlyphCluster {
    source_range: UiTextRange,
    advance: f32,
    right_to_left: bool,
}

fn glyph_cluster_at(
    glyphs: &[TextGlyph],
    start: usize,
    backend_cluster_flags: bool,
) -> Option<(GlyphCluster, usize)> {
    let first = glyphs.get(start)?;
    let mut source_range = UiTextRange {
        start: first.source_range.start,
        end: first.source_range.end,
    };
    let right_to_left = first.flags.right_to_left;
    let mut advance = 0.0;
    let mut index = start;
    while let Some(glyph) = glyphs.get(index) {
        let starts_next_cluster = if backend_cluster_flags {
            glyph.flags.cluster_start
        } else {
            glyph.source_range.start != source_range.start
                || glyph.source_range.end != source_range.end
        };
        if index > start && starts_next_cluster {
            break;
        }
        if glyph.flags.right_to_left != right_to_left {
            return None;
        }
        source_range.start = source_range.start.min(glyph.source_range.start);
        source_range.end = source_range.end.max(glyph.source_range.end);
        advance += finite_non_negative(glyph.advance);
        index += 1;
    }
    Some((
        GlyphCluster {
            source_range,
            advance,
            right_to_left,
        },
        index,
    ))
}

fn cluster_caret(
    source_range: UiTextRange,
    right_to_left: bool,
    leading_visual_edge: bool,
) -> UiTextCaret {
    let offset = if right_to_left == leading_visual_edge {
        source_range.end
    } else {
        source_range.start
    };
    UiTextCaret {
        offset,
        affinity: if leading_visual_edge {
            UiTextCaretAffinity::Downstream
        } else {
            UiTextCaretAffinity::Upstream
        },
    }
}

fn source_ranges_overlap(source_range: UiTextRange, range: UiTextRange) -> bool {
    range.start < source_range.end && source_range.start < range.end
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub(crate) fn register_resolved_text_glyph_artifact(
    artifact: Arc<ResolvedTextGlyphArtifact>,
) -> UiRichTextArtifactHandle {
    UiRichTextArtifactHandle::from_runtime_artifact(artifact)
}

pub(crate) fn resolve_resolved_text_glyph_artifact(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<ResolvedTextGlyphArtifact>> {
    handle.downcast_runtime_artifact()
}

pub(crate) fn build_resolved_text_glyph_artifact(
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    provider: &mut SharedTextLayoutSession,
) -> Option<ResolvedTextGlyphArtifact> {
    build_resolved_text_glyph_artifact_with_shared_source(
        Arc::from(source_text),
        style,
        layout,
        provider,
    )
}

/// Builds an artifact without copying a retained document's source allocation.
pub(crate) fn build_resolved_text_glyph_artifact_with_shared_source(
    source_text: Arc<str>,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    provider: &mut SharedTextLayoutSession,
) -> Option<ResolvedTextGlyphArtifact> {
    let source_text_origin = source_text_origin(source_text.as_ref(), layout.source_range);
    let shaped_style = text_style(&UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    });
    let artifact_style = UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    };
    let lines = layout
        .lines
        .iter()
        .map(|line| {
            if resolved_text_line_requires_visual_fallback(line) {
                return None;
            }
            let projected = shape_line_for_artifact(
                source_text.as_ref(),
                source_text_origin,
                &shaped_style,
                layout.writing_mode,
                line,
                provider,
            )?;
            Some(ResolvedTextGlyphArtifactLine {
                glyphs: visual_glyphs_for_line(
                    source_text.as_ref(),
                    source_text_origin,
                    line,
                    projected,
                ),
                layout_line: line.clone(),
            })
        })
        .collect::<Vec<_>>();
    lines
        .iter()
        .any(Option::is_some)
        .then(|| ResolvedTextGlyphArtifact {
            source_text,
            source_text_origin,
            font_generation: shared_font_database_generation(),
            style: artifact_style,
            writing_mode: layout.writing_mode,
            lines,
        })
}

/// Synthetic visual runs have no one-to-one source slice for artifact re-shaping. They keep the
/// resolved-layout renderer path, which shapes their actual visual text without inventing source
/// glyph ranges.
pub(crate) fn resolved_text_line_requires_visual_fallback(line: &UiResolvedTextLine) -> bool {
    line.ellipsized
        || line
            .runs
            .iter()
            .any(|run| !run.text.is_empty() && run.source_range.start == run.source_range.end)
}

pub(crate) fn rebuild_resolved_text_glyph_artifact_line(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
) -> Option<Arc<ResolvedTextGlyphArtifactLine>> {
    let line = artifact
        .lines
        .get(line_index)?
        .as_ref()?
        .layout_line
        .clone();
    let mut provider = SharedTextLayoutSession::new();
    let shaped_style = text_style(&artifact.style);
    let projected = shape_line_for_artifact(
        artifact.source_text.as_ref(),
        artifact.source_text_origin,
        &shaped_style,
        artifact.writing_mode,
        &line,
        &mut provider,
    )?;
    Some(Arc::new(ResolvedTextGlyphArtifactLine {
        glyphs: visual_glyphs_for_line(
            artifact.source_text.as_ref(),
            artifact.source_text_origin,
            &line,
            projected,
        ),
        layout_line: line,
    }))
}

fn shape_line_for_artifact(
    source_text: &str,
    source_text_origin: usize,
    style: &crate::text::TextStyle,
    writing_mode: UiTextWritingMode,
    line: &UiResolvedTextLine,
    provider: &mut SharedTextLayoutSession,
) -> Option<TextShapeResult> {
    let source = source_slice(source_text, source_text_origin, line.source_range)?;
    let shaped = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        provider.shape_vertical_line(
            source,
            style,
            line.direction.into(),
            TextRange {
                start: line.source_range.start,
                end: line.source_range.end,
            },
            VerticalMode::Mixed,
        )
    } else {
        provider.shape_horizontal_line(
            source,
            style,
            line.direction.into(),
            TextRange {
                start: line.source_range.start,
                end: line.source_range.end,
            },
        )
    };
    Some(project_shaped_glyph_run_for_runtime(shaped.as_ref()))
}

fn visual_glyphs_for_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
    shaped: TextShapeResult,
) -> Vec<TextGlyph> {
    let visual_clusters = visual_clusters_for_line(source_text, source_text_origin, line);
    let mut glyphs = shaped
        .runs
        .into_iter()
        .flat_map(|run| run.glyphs)
        .collect::<Vec<_>>();
    if visual_clusters.is_empty() {
        return glyphs;
    }

    let mut source_order = visual_clusters.clone();
    // Direct shaping is logical-order; resolve visual ranks once, then sort while retaining
    // the backend order of glyphs that share a cluster.
    source_order.sort_by(|left, right| {
        left.source_range
            .start
            .cmp(&right.source_range.start)
            .then_with(|| left.source_range.end.cmp(&right.source_range.end))
            .then_with(|| left.visual_index.cmp(&right.visual_index))
    });
    let mut projected = glyphs
        .drain(..)
        .enumerate()
        .map(|(source_index, glyph)| {
            let source_clusters = source_cluster_range_for_glyph(&source_order, &glyph);
            let visual_index = source_order[source_clusters.clone()]
                .iter()
                .map(|cluster| cluster.visual_index)
                .min()
                .unwrap_or(usize::MAX);
            ProjectedGlyph {
                glyph,
                source_index,
                visual_index,
                source_clusters,
            }
        })
        .collect::<Vec<_>>();
    projected.sort_by(|left, right| {
        left.visual_index
            .cmp(&right.visual_index)
            .then_with(|| left.source_index.cmp(&right.source_index))
    });
    apply_resolved_advances(
        &mut projected,
        source_order.as_slice(),
        line.glyph_advances.as_slice(),
        visual_clusters.len(),
    );
    projected.into_iter().map(|glyph| glyph.glyph).collect()
}

#[derive(Clone, Copy)]
struct VisualCluster {
    source_range: UiTextRange,
    visual_index: usize,
}

struct ProjectedGlyph {
    glyph: TextGlyph,
    source_index: usize,
    visual_index: usize,
    source_clusters: std::ops::Range<usize>,
}

fn visual_clusters_for_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
) -> Vec<VisualCluster> {
    if line.runs.is_empty() {
        let mut source_graphemes = source_slice(source_text, source_text_origin, line.source_range)
            .map(|source| {
                source
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| UiTextRange {
                        start: line.source_range.start + start,
                        end: line.source_range.start + start + grapheme.len(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if matches!(
            line.direction,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft
        ) {
            source_graphemes.reverse();
        }
        return line
            .text
            .grapheme_indices(true)
            .enumerate()
            .map(|(visual_index, _)| VisualCluster {
                source_range: source_graphemes
                    .get(visual_index)
                    .copied()
                    .unwrap_or(line.source_range),
                visual_index,
            })
            .collect();
    }
    let run_maps = line
        .runs
        .iter()
        .map(|run| RunSourceMap::new(source_text, source_text_origin, run))
        .collect::<Vec<_>>();
    let mut first_run = 0_usize;
    line.text
        .grapheme_indices(true)
        .enumerate()
        .map(|(visual_index, (start, grapheme))| {
            let visual_range = UiTextRange {
                start: line.visual_range.start + start,
                end: line.visual_range.start + start + grapheme.len(),
            };
            while run_maps
                .get(first_run)
                .is_some_and(|run| run.visual_range.end <= visual_range.start)
            {
                first_run += 1;
            }
            let mut source_range = None;
            for run in run_maps[first_run..]
                .iter()
                .take_while(|run| run.visual_range.start < visual_range.end)
            {
                if let Some(range) = run.source_range_for_visual(visual_range) {
                    source_range = Some(merge_ranges(source_range, range));
                }
            }
            VisualCluster {
                source_range: source_range.unwrap_or(line.source_range),
                visual_index,
            }
        })
        .collect()
}

struct RunSourceMap {
    visual_range: UiTextRange,
    visual_graphemes: Vec<UiTextRange>,
    source_graphemes: Vec<UiTextRange>,
}

impl RunSourceMap {
    fn new(
        source_text: &str,
        source_text_origin: usize,
        run: &zircon_runtime_interface::ui::surface::UiResolvedTextRun,
    ) -> Self {
        let visual_graphemes = run
            .text
            .grapheme_indices(true)
            .map(|(start, grapheme)| UiTextRange {
                start: run.visual_range.start + start,
                end: run.visual_range.start + start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        let mut source_graphemes = source_slice(source_text, source_text_origin, run.source_range)
            .map(|source| {
                source
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| UiTextRange {
                        start: run.source_range.start + start,
                        end: run.source_range.start + start + grapheme.len(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if source_graphemes.len() != visual_graphemes.len() {
            source_graphemes = vec![run.source_range; visual_graphemes.len()];
        } else if matches!(
            run.direction,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft
        ) {
            source_graphemes.reverse();
        }
        Self {
            visual_range: run.visual_range,
            visual_graphemes,
            source_graphemes,
        }
    }

    fn source_range_for_visual(&self, visual_range: UiTextRange) -> Option<UiTextRange> {
        let index = self
            .visual_graphemes
            .partition_point(|range| range.end <= visual_range.start);
        self.visual_graphemes
            .get(index)
            .filter(|range| ranges_overlap(**range, visual_range))?;
        self.source_graphemes.get(index).copied()
    }
}

fn source_cluster_range_for_glyph(
    source_order: &[VisualCluster],
    glyph: &TextGlyph,
) -> std::ops::Range<usize> {
    let start = source_order
        .partition_point(|cluster| cluster.source_range.end <= glyph.source_range.start);
    let end =
        source_order.partition_point(|cluster| cluster.source_range.start < glyph.source_range.end);
    start..end
}

fn apply_resolved_advances(
    glyphs: &mut [ProjectedGlyph],
    source_order: &[VisualCluster],
    advances: &[f32],
    cluster_count: usize,
) {
    if advances.len() != cluster_count {
        return;
    }
    for glyph in glyphs
        .iter_mut()
        .filter(|glyph| !glyph.source_clusters.is_empty())
    {
        glyph.glyph.advance = 0.0;
    }
    let mut first_glyph_by_cluster = vec![None; cluster_count];
    for (glyph_index, glyph) in glyphs.iter().enumerate() {
        for cluster in &source_order[glyph.source_clusters.clone()] {
            first_glyph_by_cluster[cluster.visual_index].get_or_insert(glyph_index);
        }
    }
    for (cluster_index, advance) in advances.iter().copied().enumerate() {
        let Some(glyph_index) = first_glyph_by_cluster[cluster_index] else {
            continue;
        };
        if advance.is_finite() {
            glyphs[glyph_index].glyph.advance += advance.max(0.0);
        }
    }
}

fn merge_ranges(current: Option<UiTextRange>, next: UiTextRange) -> UiTextRange {
    let Some(current) = current else {
        return next;
    };
    UiTextRange {
        start: current.start.min(next.start),
        end: current.end.max(next.end),
    }
}

fn ranges_overlap(left: UiTextRange, right: UiTextRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn source_text_origin(source_text: &str, layout_source_range: UiTextRange) -> usize {
    (source_text.len()
        == layout_source_range
            .end
            .saturating_sub(layout_source_range.start))
    .then_some(layout_source_range.start)
    .unwrap_or_default()
}

fn source_slice(
    source_text: &str,
    source_text_origin: usize,
    source_range: UiTextRange,
) -> Option<&str> {
    let start = source_range.start.checked_sub(source_text_origin)?;
    let end = source_range.end.checked_sub(source_text_origin)?;
    source_text.get(start..end)
}

#[cfg(test)]
mod tests;
