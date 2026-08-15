use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle,
    UiTextCaret, UiTextCaretAffinity, UiTextRange, UiTextWritingMode,
};

use super::font::{register_font_handle_batch, shared_font_database_generation};
#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
use super::font::{register_font_handle_batch_with_report, FontHandleRegistrationBatchReport};
use super::service::project_glyph;
use super::{text_style, ShapedGlyphRun, SharedTextLayoutSession, TextRange, VerticalMode};
use crate::core::framework::text::TextGlyph;

mod visual_projection;

use visual_projection::{
    apply_resolved_advances, source_cluster_range_for_glyph, visual_clusters_for_line,
    ProjectedGlyph,
};

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

struct ArtifactShapedLine {
    glyphs: Vec<TextGlyph>,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    registration_report: Option<FontHandleRegistrationBatchReport>,
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

/// A runtime glyph artifact may only retain source ranges owned by its resolved layout.
///
/// Synthetic ellipsis runs use an empty range at a line boundary and therefore remain valid.
fn artifact_line_source_ranges_are_owned_by_layout(
    layout_source_range: UiTextRange,
    line: &UiResolvedTextLine,
) -> bool {
    source_range_contains(layout_source_range, line.source_range)
        && line
            .runs
            .iter()
            .all(|run| source_range_contains(line.source_range, run.source_range))
}

fn source_range_contains(container: UiTextRange, candidate: UiTextRange) -> bool {
    container.start <= container.end
        && candidate.start <= candidate.end
        && container.start <= candidate.start
        && candidate.end <= container.end
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
    crate::profile_scope!(
        "runtime",
        "text.artifact",
        "build_resolved_text_glyph_artifact"
    );
    let collect_profile_metrics = artifact_local_profile_metrics_enabled();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let cache_report_before = collect_profile_metrics.then(|| provider.cache_report());
    let font_generation = shared_font_database_generation();
    let source_text_origin = source_text_origin(source_text.as_ref(), layout.source_range)?;
    if layout
        .lines
        .iter()
        .any(|line| !artifact_line_source_ranges_are_owned_by_layout(layout.source_range, line))
    {
        return None;
    }
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
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut registration_report = FontHandleRegistrationBatchReport::default();
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
                font_generation,
                collect_profile_metrics,
            )?;
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            if let Some(report) = projected.registration_report {
                registration_report.accumulate(report);
            }
            Some(ResolvedTextGlyphArtifactLine {
                glyphs: visual_glyphs_for_line(
                    source_text.as_ref(),
                    source_text_origin,
                    line,
                    projected.glyphs,
                ),
                layout_line: line.clone(),
            })
        })
        .collect::<Vec<_>>();
    if !lines.iter().any(Option::is_some) || shared_font_database_generation() != font_generation {
        return None;
    }
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    if collect_profile_metrics {
        let cache_report_after = provider.cache_report();
        let cache_report_before = cache_report_before
            .expect("active artifact profiling must capture the shaped-cache baseline");
        crate::profile_counter!(
            "runtime",
            "artifact_build_line_count",
            lines.iter().filter(|line| line.is_some()).count()
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_shaped_cache_hit_count",
            cache_report_after
                .hit_count
                .saturating_sub(cache_report_before.hit_count)
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_shaped_cache_miss_count",
            cache_report_after
                .miss_count
                .saturating_sub(cache_report_before.miss_count)
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_font_handle_registration_batch_count",
            registration_report.registration_batch_count
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_font_handle_registration_lock_acquire_count",
            registration_report.registration_lock_acquire_count
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_font_handle_registration_lock_wait_nanos",
            registration_report.registration_lock_wait_nanos
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_font_handle_registration_lock_hold_nanos",
            registration_report.registration_lock_hold_nanos
        );
        crate::profile_counter!(
            "runtime",
            "artifact_build_font_handle_registration_snapshot_publish_count",
            registration_report.registration_snapshot_publish_count
        );
    }
    Some(ResolvedTextGlyphArtifact {
        source_text,
        source_text_origin,
        font_generation,
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
        || !visual_runs_cover_line_in_order(line)
}

/// The artifact source-to-visual projection advances through runs with a cursor, so it must not
/// reinterpret an unordered, overlapping, or partially covered visual run collection.
fn visual_runs_cover_line_in_order(line: &UiResolvedTextLine) -> bool {
    if line.runs.is_empty() {
        return true;
    }

    let mut expected_start = line.visual_range.start;
    for run in &line.runs {
        let Some(expected_end) = run.visual_range.start.checked_add(run.text.len()) else {
            return false;
        };
        if run.visual_range.start != expected_start
            || run.visual_range.end != expected_end
            || run.visual_range.end > line.visual_range.end
        {
            return false;
        }
        expected_start = run.visual_range.end;
    }
    expected_start == line.visual_range.end
}

pub(crate) fn rebuild_resolved_text_glyph_artifact_line(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
) -> Option<(Arc<ResolvedTextGlyphArtifactLine>, u64)> {
    let line = artifact
        .lines
        .get(line_index)?
        .as_ref()?
        .layout_line
        .clone();
    let mut provider = SharedTextLayoutSession::new();
    let shaped_style = text_style(&artifact.style);
    let font_generation = shared_font_database_generation();
    let projected = shape_line_for_artifact(
        artifact.source_text.as_ref(),
        artifact.source_text_origin,
        &shaped_style,
        artifact.writing_mode,
        &line,
        &mut provider,
        font_generation,
        artifact_local_profile_metrics_enabled(),
    )?;
    let rebuilt_line = Arc::new(ResolvedTextGlyphArtifactLine {
        glyphs: visual_glyphs_for_line(
            artifact.source_text.as_ref(),
            artifact.source_text_origin,
            &line,
            projected.glyphs,
        ),
        layout_line: line,
    });
    (shared_font_database_generation() == font_generation)
        .then_some((rebuilt_line, font_generation))
}

fn shape_line_for_artifact(
    source_text: &str,
    source_text_origin: usize,
    style: &crate::text::TextStyle,
    writing_mode: UiTextWritingMode,
    line: &UiResolvedTextLine,
    provider: &mut SharedTextLayoutSession,
    font_generation: u64,
    collect_profile_metrics: bool,
) -> Option<ArtifactShapedLine> {
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
    if shared_font_database_generation() != font_generation {
        return None;
    }
    let projected =
        project_shaped_run_for_artifact(shaped.as_ref(), font_generation, collect_profile_metrics)?;
    (shared_font_database_generation() == font_generation).then_some(projected)
}

fn project_shaped_run_for_artifact(
    shaped: &ShapedGlyphRun,
    font_generation: u64,
    collect_profile_metrics: bool,
) -> Option<ArtifactShapedLine> {
    let font_pairs = shaped
        .lines
        .iter()
        .flat_map(|line| {
            line.glyphs
                .iter()
                .map(|glyph| (glyph.font_id, glyph.font_instance_id))
        })
        .collect::<Vec<_>>();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let (handles, registration_report) = if collect_profile_metrics {
        let (handles, report) =
            register_font_handle_batch_with_report(&font_pairs, font_generation);
        (handles, Some(report))
    } else {
        (
            register_font_handle_batch(&font_pairs, font_generation),
            None,
        )
    };
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let handles = {
        let _ = collect_profile_metrics;
        register_font_handle_batch(&font_pairs, font_generation)
    };
    if handles.len() != font_pairs.len() || shared_font_database_generation() != font_generation {
        return None;
    }
    Some(ArtifactShapedLine {
        glyphs: shaped
            .lines
            .iter()
            .flat_map(|line| line.glyphs.iter())
            .zip(handles)
            .map(|(glyph, handles)| project_glyph(glyph, handles))
            .collect(),
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        registration_report,
    })
}

/// Tracy streams counters continuously, while the CPU recorder should remain inert when idle.
fn artifact_local_profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}

fn visual_glyphs_for_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
    mut glyphs: Vec<TextGlyph>,
) -> Vec<TextGlyph> {
    let visual_clusters = visual_clusters_for_line(source_text, source_text_origin, line);
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

/// The artifact accepts either its complete source or precisely one absolute layout slice.
fn source_text_origin(source_text: &str, layout_source_range: UiTextRange) -> Option<usize> {
    if layout_source_range.start > layout_source_range.end {
        return None;
    }
    if layout_source_range.end <= source_text.len() {
        return Some(0);
    }
    (source_text.len() == layout_source_range.end - layout_source_range.start)
        .then_some(layout_source_range.start)
}

pub(super) fn source_slice(
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
