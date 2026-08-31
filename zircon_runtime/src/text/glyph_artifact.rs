use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiRichTextArtifactHandle,
    UiTextRange, UiTextWritingMode,
};

use super::layout::{CanonicalPhysicalLineFragment, LogicalVirtualLineSequence};
use super::{SharedTextLayoutSession, TextRange, text_style};
use crate::core::framework::text::{TextGlyph, TextLayoutError};
use crate::text::font::{
    FontCollectionSnapshot, FontHandleResolverSnapshot, font_handle_resolver_snapshot,
};
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};

mod geometry;
mod identity;
mod presentation;
mod projection;
mod rich;
mod snapshot;
mod visual_projection;

pub(crate) use geometry::{
    resolved_text_glyph_artifact_caret_advance, resolved_text_glyph_artifact_caret_at_advance,
    resolved_text_glyph_artifact_range_advance_spans,
};
use identity::ResolvedTextGlyphArtifactIdentity;
use projection::{
    artifact_local_profile_metrics_enabled, line_uses_visual_artifact_projection,
    project_shaped_run_for_artifact, shape_line_for_artifact,
    shape_logical_virtual_line_for_artifact, shape_visual_line_for_artifact,
};
pub(crate) use snapshot::{
    resolved_text_glyph_artifact_line_matches_layout,
    resolved_text_glyph_artifact_matches_layout_snapshot,
};
use visual_projection::{
    ProjectedGlyph, apply_resolved_advances, source_cluster_range_for_glyph,
    visual_clusters_for_line,
};

pub(crate) use presentation::build_resolved_text_presentation_glyph_artifact;
pub(crate) use rich::{
    BuiltResolvedRichTextGlyphArtifact, build_resolved_rich_text_glyph_artifact,
};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTextGlyphArtifactFontLease {
    font_collection: FontCollectionSnapshot,
    font_handles: FontHandleResolverSnapshot,
}

impl ResolvedTextGlyphArtifactFontLease {
    pub(crate) fn capture(font_collection: FontCollectionSnapshot) -> Self {
        let font_handles = font_handle_resolver_snapshot(&font_collection);
        Self {
            font_collection,
            font_handles,
        }
    }

    pub(crate) fn revision(&self) -> crate::text::font::FontCollectionRevision {
        self.font_collection.revision()
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.font_collection.generation()
    }

    pub(crate) const fn font_collection(&self) -> &FontCollectionSnapshot {
        &self.font_collection
    }

    pub(crate) const fn font_handles(&self) -> &FontHandleResolverSnapshot {
        &self.font_handles
    }

    #[cfg(test)]
    pub(crate) fn process_default() -> Self {
        Self::capture(crate::text::font::shared_font_collection_snapshot())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedTextGlyphArtifact {
    pub(crate) source_text: Arc<str>,
    pub(crate) source_text_origin: usize,
    pub(crate) font_generation: u64,
    pub(crate) font_lease: ResolvedTextGlyphArtifactFontLease,
    pub(crate) style: UiResolvedStyle,
    pub(crate) writing_mode: UiTextWritingMode,
    pub(crate) lines: Vec<Option<ResolvedTextGlyphArtifactLine>>,
    /// Process-local logical display inputs needed to rebuild generated RTL/mixed visual lines.
    /// This is layout-owner state and deliberately never crosses the UI DTO boundary.
    pub(crate) logical_virtual_line_sequences: Option<Vec<Option<LogicalVirtualLineSequence>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedTextGlyphArtifactLine {
    pub(crate) glyphs: Vec<TextGlyph>,
    pub(crate) layout_line: UiResolvedTextLine,
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

/// Source ranges are admitted to visual projection only when they can be sliced from the same
/// source snapshot used by the layout. Empty ranges are valid virtual anchors, but still must land
/// on a UTF-8 boundary in that snapshot.
fn artifact_line_source_ranges_are_sliceable(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
) -> bool {
    let range_is_sliceable =
        |range: UiTextRange| source_slice(source_text, source_text_origin, range).is_some();
    range_is_sliceable(line.source_range)
        && line
            .runs
            .iter()
            .all(|run| range_is_sliceable(run.source_range))
}

fn source_range_contains(container: UiTextRange, candidate: UiTextRange) -> bool {
    container.start <= container.end
        && candidate.start <= candidate.end
        && container.start <= candidate.start
        && candidate.end <= container.end
}

pub(crate) fn register_resolved_text_glyph_artifact(
    artifact: Arc<ResolvedTextGlyphArtifact>,
) -> UiRichTextArtifactHandle {
    let identity = ResolvedTextGlyphArtifactIdentity::new(Arc::clone(&artifact));
    UiRichTextArtifactHandle::from_runtime_artifact_with_identity(artifact, identity)
}

pub(crate) fn resolve_resolved_text_glyph_artifact(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<ResolvedTextGlyphArtifact>> {
    handle
        .downcast_runtime_artifact()
        .or_else(|| super::runtime_artifact::resolve_text_glyphs_from_composite(handle))
}

pub(crate) fn build_resolved_text_glyph_artifact(
    source_text: &str,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<ResolvedTextGlyphArtifact>> {
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
) -> TextLayoutOutcome<Option<ResolvedTextGlyphArtifact>> {
    build_resolved_text_glyph_artifact_with_line_fragments(
        source_text,
        style,
        layout,
        None,
        None,
        provider,
    )
}

/// Projects optional retained final-line fragments into the renderer artifact without reshaping
/// their source-congruent lines.
///
/// The fragments are private layout-owner state, deliberately separate from the serializable UI
/// layout DTO. A missing or non-matching entry retains the established artifact shaping path.
pub(crate) fn build_resolved_text_glyph_artifact_with_line_fragments(
    source_text: Arc<str>,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    retained_line_fragments: Option<&[Option<Arc<CanonicalPhysicalLineFragment>>]>,
    retained_virtual_line_sequences: Option<&[Option<LogicalVirtualLineSequence>]>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<ResolvedTextGlyphArtifact>> {
    crate::profile_scope!(
        "runtime",
        "text.artifact",
        "build_resolved_text_glyph_artifact"
    );
    let collect_profile_metrics = artifact_local_profile_metrics_enabled();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let cache_report_before = collect_profile_metrics.then(|| provider.cache_report());
    let font_collection = provider.font_collection_snapshot();
    let font_revision = font_collection.revision();
    let font_generation = font_collection.generation();
    let Some(source_text_origin) = source_text_origin(source_text.as_ref(), layout.source_range)
    else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    if layout.lines.iter().any(|line| {
        !artifact_line_source_ranges_are_owned_by_layout(layout.source_range, line)
            || !artifact_line_source_ranges_are_sliceable(
                source_text.as_ref(),
                source_text_origin,
                line,
            )
    }) {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
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
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut retained_fragment_projection_count = 0_usize;
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut fallback_shape_request_count = 0_usize;
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut visual_projection_shape_request_count = 0_usize;
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut logical_virtual_projection_shape_request_count = 0_usize;
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut retained_logical_virtual_fragment_projection_count = 0_usize;
    let mut lines = Vec::with_capacity(layout.lines.len());
    for (line_index, line) in layout.lines.iter().enumerate() {
        let logical_virtual_sequence = retained_virtual_line_sequences
            .and_then(|sequences| sequences.get(line_index))
            .and_then(Option::as_ref);
        let virtual_renderer_fallback = logical_virtual_sequence
            .is_some_and(|sequence| !sequence.artifact_projection_allowed());
        let uses_logical_virtual_projection =
            matches!(layout.writing_mode, UiTextWritingMode::HorizontalTb)
                && logical_virtual_sequence
                    .is_some_and(|sequence| sequence.artifact_projection_allowed());
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let reuses_logical_virtual_fragment = logical_virtual_sequence
            .and_then(|sequence| sequence.fragment_for_revision(font_revision))
            .is_some();
        let uses_visual_projection = !virtual_renderer_fallback
            && !uses_logical_virtual_projection
            && line_uses_visual_artifact_projection(layout.writing_mode, line);
        if virtual_renderer_fallback
            || (!uses_logical_virtual_projection
                && !uses_visual_projection
                && resolved_text_line_requires_visual_fallback(line))
        {
            lines.push(None);
            continue;
        }
        let projection_outcome = if let Some(sequence) =
            logical_virtual_sequence.filter(|_| uses_logical_virtual_projection)
        {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            {
                if reuses_logical_virtual_fragment {
                    retained_logical_virtual_fragment_projection_count =
                        retained_logical_virtual_fragment_projection_count.saturating_add(1);
                } else {
                    logical_virtual_projection_shape_request_count =
                        logical_virtual_projection_shape_request_count.saturating_add(1);
                }
            }
            shape_logical_virtual_line_for_artifact(
                sequence,
                &shaped_style,
                line,
                provider,
                &font_collection,
                collect_profile_metrics,
            )
        } else if uses_visual_projection {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            {
                visual_projection_shape_request_count =
                    visual_projection_shape_request_count.saturating_add(1);
            }
            shape_visual_line_for_artifact(
                source_text.as_ref(),
                source_text_origin,
                &shaped_style,
                line,
                provider,
                &font_collection,
                collect_profile_metrics,
            )
        } else if let Some(fragment) = retained_line_fragment_for_artifact(
            retained_line_fragments,
            line_index,
            source_text.as_ref(),
            source_text_origin,
            font_revision,
            line,
        ) {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            {
                retained_fragment_projection_count =
                    retained_fragment_projection_count.saturating_add(1);
            }
            project_shaped_run_for_artifact(
                fragment.shaped().as_ref(),
                &font_collection,
                collect_profile_metrics,
            )
            .map(Some)
        } else {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            {
                fallback_shape_request_count = fallback_shape_request_count.saturating_add(1);
            }
            shape_line_for_artifact(
                source_text.as_ref(),
                source_text_origin,
                &shaped_style,
                layout.writing_mode,
                line,
                provider,
                &font_collection,
                collect_profile_metrics,
            )
        };
        let projected = match projection_outcome {
            TextShapingOutcome::Ready(Some(projected)) => projected,
            TextShapingOutcome::Ready(None) => {
                lines.push(None);
                continue;
            }
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        if let Some(report) = projected.registration_report {
            registration_report.accumulate(report);
        }
        lines.push(Some(ResolvedTextGlyphArtifactLine {
            glyphs: if uses_logical_virtual_projection || uses_visual_projection {
                projected.glyphs
            } else {
                visual_glyphs_for_line(
                    source_text.as_ref(),
                    source_text_origin,
                    line,
                    projected.glyphs,
                )
            },
            layout_line: line.clone(),
        }));
    }
    if !lines.iter().any(Option::is_some) {
        return TextShapingOutcome::Ready(None);
    }
    if font_collection.service().generation() != font_generation {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    if collect_profile_metrics {
        if let Some(cache_report_before) = cache_report_before {
            let cache_report_after = provider.cache_report();
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
                "artifact_build_retained_fragment_projection_count",
                retained_fragment_projection_count
            );
            crate::profile_counter!(
                "runtime",
                "artifact_build_fallback_shape_request_count",
                fallback_shape_request_count
            );
            crate::profile_counter!(
                "runtime",
                "artifact_build_visual_projection_shape_request_count",
                visual_projection_shape_request_count
            );
            crate::profile_counter!(
                "runtime",
                "artifact_build_logical_virtual_projection_shape_request_count",
                logical_virtual_projection_shape_request_count
            );
            crate::profile_counter!(
                "runtime",
                "artifact_build_retained_logical_virtual_fragment_projection_count",
                retained_logical_virtual_fragment_projection_count
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
    }
    TextShapingOutcome::Ready(Some(ResolvedTextGlyphArtifact {
        source_text,
        source_text_origin,
        font_generation,
        font_lease: ResolvedTextGlyphArtifactFontLease::capture(font_collection),
        style: artifact_style,
        writing_mode: layout.writing_mode,
        lines,
        logical_virtual_line_sequences: retained_virtual_line_sequences
            .map(|sequences| sequences.to_vec()),
    }))
}

fn retained_line_fragment_for_artifact<'a>(
    retained_line_fragments: Option<&'a [Option<Arc<CanonicalPhysicalLineFragment>>]>,
    line_index: usize,
    source_text: &str,
    source_text_origin: usize,
    font_revision: crate::text::font::FontCollectionRevision,
    line: &UiResolvedTextLine,
) -> Option<&'a CanonicalPhysicalLineFragment> {
    let fragment = retained_line_fragments?.get(line_index)?.as_deref()?;
    let source = source_slice(source_text, source_text_origin, line.source_range)?;
    (fragment.shaped().source_range
        == TextRange {
            start: line.source_range.start,
            end: line.source_range.end,
        }
        && fragment.font_collection_revision() == font_revision
        && fragment.shaped().source_text.as_ref() == source)
        .then_some(fragment)
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
pub(in crate::text) fn source_text_origin(
    source_text: &str,
    layout_source_range: UiTextRange,
) -> Option<usize> {
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
