use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextRange, UiTextWritingMode,
};

use crate::core::framework::text::{TextDirection, TextGlyph, TextLayoutError};
use crate::text::layout::{
    LogicalVirtualFragmentRole, LogicalVirtualLineSequence, ResolvedRichTextSpan,
    RichTextLayoutSource, resolved_text_spans,
};
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{
    ResolvedRichTextGlyphRun, SharedTextLayoutSession, TextRange, TextStyle, VerticalMode,
    text_style,
};

use super::projection::{artifact_local_profile_metrics_enabled, project_shaped_runs_for_artifact};
use super::visual_projection::{source_cluster_range_for_glyph, visual_clusters_for_line};
use super::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactFontLease, ResolvedTextGlyphArtifactLine,
    artifact_line_source_ranges_are_owned_by_layout, artifact_line_source_ranges_are_sliceable,
    resolved_text_line_requires_visual_fallback, source_slice, source_text_origin,
    visual_glyphs_for_line,
};

pub(crate) struct BuiltResolvedRichTextGlyphArtifact {
    pub(crate) artifact: Arc<ResolvedTextGlyphArtifact>,
    pub(crate) glyph_runs: Arc<[ResolvedRichTextGlyphRun]>,
}

pub(crate) fn build_resolved_rich_text_glyph_artifact<S>(
    source: &S,
    source_text: Arc<str>,
    style: &UiResolvedStyle,
    layout: &UiResolvedTextLayout,
    retained_virtual_line_sequences: Option<&[Option<LogicalVirtualLineSequence>]>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<BuiltResolvedRichTextGlyphArtifact>>
where
    S: RichTextLayoutSource + ?Sized,
{
    crate::profile_scope!(
        "runtime",
        "text.artifact",
        "build_resolved_rich_text_glyph_artifact"
    );
    let Some(source_text_origin) = source_text_origin(source_text.as_ref(), layout.source_range)
    else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    if source.text() != source_text.as_ref()
        || layout.lines.iter().any(|line| {
            !artifact_line_source_ranges_are_owned_by_layout(layout.source_range, line)
                || !artifact_line_source_ranges_are_sliceable(
                    source_text.as_ref(),
                    source_text_origin,
                    line,
                )
        })
    {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    }

    let collect_profile_metrics = artifact_local_profile_metrics_enabled();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let cache_report_before = collect_profile_metrics.then(|| provider.cache_report());
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut shape_request_count = 0_usize;
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut shape_source_bytes = 0_usize;
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut logical_sidecar_line_count = 0_usize;
    let font_collection = provider.font_collection_snapshot();
    let font_revision = font_collection.revision();
    let font_generation = font_collection.generation();
    let artifact_style = UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        ..style.clone()
    };
    let spans = match resolved_text_spans(source, &text_style(&artifact_style)) {
        Ok(spans) => spans,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    let mut lines = Vec::with_capacity(layout.lines.len());
    let mut glyph_runs = Vec::new();

    for (line_index, line) in layout.lines.iter().enumerate() {
        let logical_virtual_sequence = retained_virtual_line_sequences
            .and_then(|sequences| sequences.get(line_index))
            .and_then(Option::as_ref)
            .filter(|sequence| sequence.artifact_projection_allowed())
            .filter(|sequence| {
                matches!(layout.writing_mode, UiTextWritingMode::HorizontalTb)
                    || vertical_sequence_uses_supported_generated_markers(sequence)
            });
        let external_only_line = logical_virtual_sequence.is_some_and(|sequence| {
            let mut receipts = sequence.logical_cluster_receipts();
            receipts
                .next()
                .is_some_and(|(_, _, _, _, external)| external)
                && receipts.all(|(_, _, _, _, external)| external)
        });
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        if logical_virtual_sequence.is_some() {
            logical_sidecar_line_count = logical_sidecar_line_count.saturating_add(1);
        }
        if logical_virtual_sequence.is_none()
            && (resolved_text_line_requires_visual_fallback(line)
                || line_contains_source_owned_soft_hyphen_marker(
                    source_text.as_ref(),
                    source_text_origin,
                    line,
                ))
        {
            lines.push(None);
            continue;
        }
        let mut shaped_runs = Vec::new();
        if let Some(sequence) = logical_virtual_sequence {
            let logical_spans = match logical_virtual_style_spans(sequence, spans.as_slice()) {
                Some(spans) => spans,
                None if external_only_line => Vec::new(),
                None => {
                    lines.push(None);
                    continue;
                }
            };
            for span in logical_spans {
                let Some(text) = sequence.text().get(span.range.start..span.range.end) else {
                    return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
                };
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                {
                    shape_request_count = shape_request_count.saturating_add(1);
                    shape_source_bytes = shape_source_bytes.saturating_add(text.len());
                }
                match shape_rich_span_for_artifact(
                    text,
                    &span.style,
                    sequence.base_direction(),
                    span.range,
                    layout.writing_mode,
                    provider,
                ) {
                    TextShapingOutcome::Ready(shaped) => shaped_runs.push(shaped),
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => {
                        return TextShapingOutcome::Failed(error);
                    }
                }
            }
        } else {
            for span in spans.iter().filter(|span| {
                span.start < line.source_range.end && line.source_range.start < span.end
            }) {
                let range = TextRange {
                    start: span.start.max(line.source_range.start),
                    end: span.end.min(line.source_range.end),
                };
                let Some(text) = source_slice(
                    source_text.as_ref(),
                    source_text_origin,
                    UiTextRange {
                        start: range.start,
                        end: range.end,
                    },
                ) else {
                    return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
                };
                if text.is_empty() {
                    continue;
                }
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                {
                    shape_request_count = shape_request_count.saturating_add(1);
                    shape_source_bytes = shape_source_bytes.saturating_add(text.len());
                }
                let direction = if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
                    line.direction.into()
                } else {
                    TextDirection::Auto
                };
                match shape_rich_span_for_artifact(
                    text,
                    &span.style,
                    direction,
                    range,
                    layout.writing_mode,
                    provider,
                ) {
                    TextShapingOutcome::Ready(shaped) => shaped_runs.push(shaped),
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => {
                        return TextShapingOutcome::Failed(error);
                    }
                }
            }
        }
        if shaped_runs.is_empty() && !external_only_line {
            lines.push(None);
            continue;
        }
        if provider.font_collection_revision() != font_revision {
            return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
        }
        let glyphs = if external_only_line {
            Vec::new()
        } else {
            let projected = match project_shaped_runs_for_artifact(
                shaped_runs.as_slice(),
                &font_collection,
                collect_profile_metrics,
            ) {
                TextShapingOutcome::Ready(projected) => projected,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            if let Some(sequence) = logical_virtual_sequence {
                let Some(glyphs) =
                    sequence.project_logical_glyphs(projected.glyphs, &line.glyph_advances)
                else {
                    lines.push(None);
                    continue;
                };
                glyphs
            } else {
                visual_glyphs_for_line(
                    source_text.as_ref(),
                    source_text_origin,
                    line,
                    projected.glyphs,
                )
            }
        };
        let Some(mut line_glyph_runs) = glyph_run_ranges(
            source_text.as_ref(),
            source_text_origin,
            line_index,
            line,
            glyphs.as_slice(),
            logical_virtual_sequence,
        ) else {
            lines.push(None);
            continue;
        };
        if (!external_only_line && glyphs.is_empty()) || line_glyph_runs.is_empty() {
            lines.push(None);
            continue;
        }
        glyph_runs.append(&mut line_glyph_runs);
        lines.push(Some(ResolvedTextGlyphArtifactLine {
            glyphs,
            layout_line: line.clone(),
        }));
    }

    if provider.font_collection_revision() != font_revision {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    if collect_profile_metrics {
        crate::profile_counter!(
            "runtime",
            "rich_artifact_shape_request_count",
            shape_request_count
        );
        crate::profile_counter!(
            "runtime",
            "rich_artifact_shape_source_bytes",
            shape_source_bytes
        );
        crate::profile_counter!(
            "runtime",
            "rich_artifact_mapped_run_count",
            glyph_runs.len()
        );
        crate::profile_counter!(
            "runtime",
            "rich_artifact_logical_sidecar_line_count",
            logical_sidecar_line_count
        );
        if let Some(cache_report_before) = cache_report_before {
            let cache_report_after = provider.cache_report();
            crate::profile_counter!(
                "runtime",
                "rich_artifact_shaped_cache_hit_count",
                cache_report_after
                    .hit_count
                    .saturating_sub(cache_report_before.hit_count)
            );
            crate::profile_counter!(
                "runtime",
                "rich_artifact_shaped_cache_miss_count",
                cache_report_after
                    .miss_count
                    .saturating_sub(cache_report_before.miss_count)
            );
        }
    }
    TextShapingOutcome::Ready(Some(BuiltResolvedRichTextGlyphArtifact {
        artifact: Arc::new(ResolvedTextGlyphArtifact {
            source_text,
            source_text_origin,
            font_generation,
            font_lease: ResolvedTextGlyphArtifactFontLease::capture(font_collection),
            style: artifact_style,
            writing_mode: layout.writing_mode,
            lines,
            logical_virtual_line_sequences: retained_virtual_line_sequences
                .map(|sequences| sequences.to_vec()),
        }),
        glyph_runs: Arc::from(glyph_runs),
    }))
}

struct LogicalVirtualStyleSpan {
    range: TextRange,
    style: TextStyle,
}

fn shape_rich_span_for_artifact<P>(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    writing_mode: UiTextWritingMode,
    provider: &mut P,
) -> TextShapingOutcome
where
    P: TextShapeRunProvider + ?Sized,
{
    if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        provider.shape_vertical_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            VerticalMode::Mixed,
            true,
        )
    } else {
        provider.shape_horizontal_range_with_kerning(text, style, direction, source_range, true)
    }
}

fn vertical_sequence_uses_supported_generated_markers(
    sequence: &LogicalVirtualLineSequence,
) -> bool {
    sequence.logical_virtual_fragment_roles().all(
        |(logical_range, source_range, replaced_source_range, virtual_role)| {
            if source_range.start < source_range.end {
                return virtual_role.is_none();
            }
            let marker = sequence.text().get(logical_range.start..logical_range.end);
            match virtual_role {
                Some(LogicalVirtualFragmentRole::Ellipsis) => marker == Some("\u{2026}"),
                Some(LogicalVirtualFragmentRole::DiscretionaryHyphen) => {
                    marker == Some("-") && replaced_source_range.is_some()
                }
                Some(LogicalVirtualFragmentRole::Justification) | None => false,
            }
        },
    )
}

fn logical_virtual_style_spans(
    sequence: &LogicalVirtualLineSequence,
    source_spans: &[ResolvedRichTextSpan],
) -> Option<Vec<LogicalVirtualStyleSpan>> {
    let mut result = Vec::<LogicalVirtualStyleSpan>::new();
    let mut source_span_index = 0_usize;
    for (logical_range, source_range, style_owner_source_range, _, external) in
        sequence.logical_cluster_receipts()
    {
        if external {
            continue;
        }
        let style_source_range = style_owner_source_range.unwrap_or(source_range);
        while source_spans.get(source_span_index).is_some_and(|span| {
            !source_cluster_belongs_to_span(style_source_range, span)
                && span.end <= style_source_range.start
        }) {
            source_span_index = source_span_index.saturating_add(1);
        }
        let span = source_spans.get(source_span_index)?;
        if !source_cluster_belongs_to_span(style_source_range, span) {
            return None;
        }
        if let Some(previous) = result.last_mut() {
            if previous.range.end == logical_range.start && previous.style == span.style {
                previous.range.end = logical_range.end;
                continue;
            }
        }
        result.push(LogicalVirtualStyleSpan {
            range: logical_range,
            style: span.style.clone(),
        });
    }
    (!result.is_empty()).then_some(result)
}

fn source_cluster_belongs_to_span(cluster: TextRange, span: &ResolvedRichTextSpan) -> bool {
    if cluster.start == cluster.end {
        (cluster.start == 0 && span.start == 0)
            || (span.start < cluster.start && cluster.start <= span.end)
    } else {
        span.start <= cluster.start && cluster.end <= span.end
    }
}

fn line_contains_source_owned_soft_hyphen_marker(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
) -> bool {
    line.runs.iter().any(|run| {
        source_slice(source_text, source_text_origin, run.source_range)
            .is_some_and(|source| source.contains('\u{00ad}') && source != run.text)
    })
}

fn glyph_run_ranges(
    source_text: &str,
    source_text_origin: usize,
    line_index: usize,
    line: &UiResolvedTextLine,
    glyphs: &[TextGlyph],
    logical_virtual_sequence: Option<&LogicalVirtualLineSequence>,
) -> Option<Vec<ResolvedRichTextGlyphRun>> {
    let visual_clusters = visual_clusters_for_line(source_text, source_text_origin, line);
    if visual_clusters.is_empty() {
        return None;
    }
    let mut source_order = visual_clusters.clone();
    source_order.sort_by(|left, right| {
        left.source_range
            .start
            .cmp(&right.source_range.start)
            .then_with(|| left.source_range.end.cmp(&right.source_range.end))
            .then_with(|| left.visual_index.cmp(&right.visual_index))
    });
    let mut ranges: Vec<Option<std::ops::Range<usize>>> = vec![None; line.runs.len()];
    let mut style_source_ranges: Vec<Option<UiTextRange>> = vec![None; line.runs.len()];
    let mut replaced_source_ranges: Vec<Option<UiTextRange>> = vec![None; line.runs.len()];
    let visual_source_receipts =
        logical_virtual_sequence.map(LogicalVirtualLineSequence::visual_source_receipts);
    let mut virtual_visual_cursor = 0_usize;
    for (glyph_index, glyph) in glyphs.iter().enumerate() {
        let visual_index = if glyph.flags.virtual_glyph {
            let relative_index =
                visual_clusters[virtual_visual_cursor..]
                    .iter()
                    .position(|cluster| {
                        cluster.source_range.start == glyph.source_range.start
                            && cluster.source_range.end == glyph.source_range.end
                    })?;
            let visual_index = virtual_visual_cursor.saturating_add(relative_index);
            virtual_visual_cursor = visual_index.saturating_add(1);
            visual_index
        } else {
            let source_clusters = source_cluster_range_for_glyph(source_order.as_slice(), glyph);
            source_order[source_clusters]
                .iter()
                .map(|cluster| cluster.visual_index)
                .min()?
        };
        let visual_range = visual_clusters.get(visual_index)?.visual_range;
        let owner_index = line.runs.iter().position(|run| {
            run.visual_range.start <= visual_range.start && visual_range.end <= run.visual_range.end
        })?;
        if glyph.flags.virtual_glyph {
            let source_receipt = visual_source_receipts
                .as_ref()
                .and_then(|ranges| ranges.get(visual_index))
                .copied()
                .flatten();
            let style_source_range =
                source_receipt.map(|receipt| UiTextRange::from(receipt.style_source_range));
            match (style_source_ranges[owner_index], style_source_range) {
                (Some(existing), Some(current)) if existing != current => return None,
                (None, Some(current)) => style_source_ranges[owner_index] = Some(current),
                _ => {}
            }
            let replaced_source_range = source_receipt
                .and_then(|receipt| receipt.replaced_source_range)
                .map(Into::into);
            match (replaced_source_ranges[owner_index], replaced_source_range) {
                (Some(existing), Some(current)) if existing != current => return None,
                (None, Some(current)) => replaced_source_ranges[owner_index] = Some(current),
                _ => {}
            }
        }
        match &mut ranges[owner_index] {
            Some(range) if range.end == glyph_index => range.end = glyph_index + 1,
            Some(_) => return None,
            slot @ None => *slot = Some(glyph_index..glyph_index + 1),
        }
    }
    let mut glyph_cursor = 0_usize;
    Some(
        line.runs
            .iter()
            .zip(ranges)
            .zip(style_source_ranges)
            .zip(replaced_source_ranges)
            .map(
                |(((run, glyph_range), style_source_range), replaced_source_range)| {
                    let glyph_range = glyph_range.unwrap_or(glyph_cursor..glyph_cursor);
                    glyph_cursor = glyph_cursor.max(glyph_range.end);
                    ResolvedRichTextGlyphRun {
                        line_index,
                        source_range: run.source_range,
                        visual_range: run.visual_range,
                        style_source_range,
                        replaced_source_range,
                        glyph_range,
                    }
                },
            )
            .collect(),
    )
}

#[cfg(test)]
#[path = "tests/rich_builder.rs"]
mod tests;
