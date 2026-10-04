use std::sync::Arc;

use crate::text::{text_glyph_clusters, TextSize, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::text::layout_geometry::{finite_sum, FiniteGeometryAccumulator};
use crate::text::shaping::{
    DirectTextShapeRunProvider, TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome,
};
use crate::text::TextRange;
use crate::text::{ShapedGlyph, ShapedGlyphBreakSafety, ShapedGlyphRun};

use super::tab::tab_aligned_width_for_matching_graphemes;

mod grapheme_projection;
use grapheme_projection::{
    aligned_grapheme_span, validate_shaped_geometry_source, GraphemeProjectionCursor,
};

const DEFAULT_METRICS_SAMPLE: &str = "Hg";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLineMetrics {
    pub width: f32,
    pub baseline: f32,
    pub line_height: f32,
}

/// Metrics, advances, and identity retained from one final physical-line shape request.
///
/// Text03 line boxes need both views of the same shaped glyph run. Keeping them
/// together retains the canonical input for artifact projection instead of requiring a
/// metrics-only or artifact-only re-shape.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeasuredTextLine {
    pub(crate) shaped: Arc<ShapedGlyphRun>,
    pub(crate) metrics: TextLineMetrics,
    pub(crate) grapheme_advances: Vec<f32>,
    pub(crate) glyph_clusters: Vec<MeasuredGlyphCluster>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MeasuredClusterCaretPolicy {
    GraphemeBoundary,
    AtomicCluster,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredGlyphCluster {
    pub(crate) source_range: TextRange,
    pub(crate) advance: f32,
    pub(crate) caret_policy: MeasuredClusterCaretPolicy,
    pub(crate) break_safety: ShapedGlyphBreakSafety,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeasuredGraphemeGeometry {
    pub(crate) advances: Vec<f32>,
    pub(crate) glyph_clusters: Vec<MeasuredGlyphCluster>,
}

pub(crate) fn measure_text_size(text: &str, style: &TextStyle) -> TextLayoutOutcome<TextSize> {
    let mut provider = DirectTextShapeRunProvider::default();
    measure_text_size_with_provider(text, style, &mut provider)
}

pub(crate) fn measure_text_size_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<TextSize>
where
    P: TextShapeRunProvider + ?Sized,
{
    let mut width = 0.0_f32;
    let mut height = FiniteGeometryAccumulator::default();
    let mut line_count = 0_usize;
    let mut empty_line_metrics = None;
    for hard_line in crate::text::hard_lines(text) {
        let Some(line) = text.get(hard_line.content) else {
            return TextShapingOutcome::failed(
                crate::core::framework::text::TextLayoutError::LayoutFailed,
            );
        };
        let (line_width, line_height) = if line.is_empty() {
            let metrics = match empty_line_metrics {
                Some(metrics) => metrics,
                None => match line_metrics_with_provider(style, provider) {
                    TextShapingOutcome::Ready(metrics) => {
                        empty_line_metrics = Some(metrics);
                        metrics
                    }
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                },
            };
            (0.0, metrics.line_height)
        } else {
            let (measured, contains_tab) = match measure_line_with_tab_presence_with_provider::<
                true,
                P,
            >(line, style, provider)
            {
                TextShapingOutcome::Ready(measured) => measured,
                TextShapingOutcome::Deferred(error) => {
                    return TextShapingOutcome::Deferred(error);
                }
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            let line_width = match measure_line_width_from_shaped_with_provider(
                line,
                contains_tab,
                style,
                &measured,
                provider,
            ) {
                TextShapingOutcome::Ready(width) => width,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            (line_width, measured.metrics.line_height)
        };
        width = width.max(line_width);
        height.add(line_height);
        line_count = line_count.saturating_add(1);
    }
    if line_count == 0 {
        let metrics = match line_metrics_with_provider(style, provider) {
            TextShapingOutcome::Ready(metrics) => metrics,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        return TextShapingOutcome::Ready(TextSize::new(width, metrics.line_height));
    }
    TextShapingOutcome::Ready(TextSize::new(width, height.value()))
}

pub(crate) fn measure_line_width(text: &str, style: &TextStyle) -> TextLayoutOutcome<f32> {
    let mut provider = DirectTextShapeRunProvider::default();
    measure_line_width_with_provider(text, style, &mut provider)
}

pub(crate) fn measure_line_width_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<f32>
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() {
        return TextShapingOutcome::Ready(0.0);
    }

    measure_line_with_tab_presence_with_provider::<true, P>(text, style, provider).and_then(
        |(measured, contains_tab)| {
            measure_line_width_from_shaped_with_provider(
                text,
                contains_tab,
                style,
                &measured,
                provider,
            )
        },
    )
}

pub(crate) fn measured_grapheme_widths(
    text: &str,
    style: &TextStyle,
) -> TextLayoutOutcome<Vec<f32>> {
    let mut provider = DirectTextShapeRunProvider::default();
    measured_grapheme_widths_with_provider(text, style, &mut provider)
}

pub(crate) fn measured_grapheme_widths_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<f32>>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_unconstrained_line_with_provider(text, style, provider).and_then(|shaped| {
        TextShapingOutcome::from_result(measured_grapheme_widths_from_shaped(&shaped, text))
    })
}

pub(crate) fn measured_grapheme_geometry_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<MeasuredGraphemeGeometry>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_unconstrained_line_with_provider(text, style, provider).and_then(|shaped| {
        TextShapingOutcome::from_result(measured_grapheme_geometry_from_shaped(&shaped, text))
    })
}

/// Shapes one unwrapped physical-line candidate once and exposes both metrics
/// needed by line-box placement and advances needed by width/selection logic.
pub(crate) fn measure_line_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<MeasuredTextLine>
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_line_with_tab_presence_with_provider::<false, P>(text, style, provider)
        .map(|(measured, _)| measured)
}

fn measure_line_with_tab_presence_with_provider<const TRACK_TAB: bool, P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<(MeasuredTextLine, bool)>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_unconstrained_line_with_provider(text, style, provider).and_then(|shaped| {
        TextShapingOutcome::from_result(project_grapheme_geometry_with_tab_presence_from_shaped::<
            TRACK_TAB,
        >(&shaped, text, true))
        .map(|(geometry, contains_tab)| {
            (
                MeasuredTextLine {
                    metrics: text_line_metrics_from_shaped(&shaped, style),
                    grapheme_advances: geometry.advances,
                    glyph_clusters: geometry.glyph_clusters,
                    shaped,
                },
                contains_tab,
            )
        })
    })
}

fn measure_line_width_from_shaped_with_provider<P>(
    text: &str,
    contains_tab: bool,
    style: &TextStyle,
    measured: &MeasuredTextLine,
    provider: &mut P,
) -> TextLayoutOutcome<f32>
where
    P: TextShapeRunProvider + ?Sized,
{
    if !contains_tab {
        return TextShapingOutcome::Ready(measured.metrics.width);
    }

    shape_line_with_provider(" ", style, provider).map(|space_metrics| {
        tab_aligned_width_for_matching_graphemes(
            text,
            &measured.grapheme_advances,
            style,
            space_metrics.width,
        )
    })
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &TextStyle,
    range: TextRange,
) -> TextLayoutOutcome<f32> {
    let mut provider = DirectTextShapeRunProvider::default();
    measure_text_source_range_width_with_provider(text, style, range, &mut provider)
}

pub(crate) fn measure_text_source_range_width_with_provider<P>(
    text: &str,
    style: &TextStyle,
    range: TextRange,
    provider: &mut P,
) -> TextLayoutOutcome<f32>
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_text_source_range_width_with_kerning_and_provider(text, style, range, true, provider)
}

#[cfg(test)]
pub(crate) fn measure_text_source_range_width_with_kerning(
    text: &str,
    style: &TextStyle,
    range: TextRange,
    include_kerning: bool,
) -> TextLayoutOutcome<f32> {
    let mut provider = DirectTextShapeRunProvider::default();
    measure_text_source_range_width_with_kerning_and_provider(
        text,
        style,
        range,
        include_kerning,
        &mut provider,
    )
}

pub(crate) fn measure_text_source_range_width_with_kerning_and_provider<P>(
    text: &str,
    style: &TextStyle,
    range: TextRange,
    include_kerning: bool,
    provider: &mut P,
) -> TextLayoutOutcome<f32>
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() || range.start >= range.end {
        return TextShapingOutcome::Ready(0.0);
    }

    shape_unconstrained_line_with_kerning_and_provider(text, style, include_kerning, provider)
        .map(|shaped| measured_width(&shaped, range.start, range.end, include_kerning))
}

pub(crate) fn measured_width(
    run: &ShapedGlyphRun,
    byte_start: usize,
    byte_end: usize,
    include_kerning: bool,
) -> f32 {
    if byte_start >= byte_end {
        return 0.0;
    }
    if validate_shaped_geometry_source(run, run.source_text.as_ref()).is_err() {
        return 0.0;
    }
    let Some(relative_start) = byte_start.checked_sub(run.source_range.start) else {
        return 0.0;
    };
    let Some(relative_end) = byte_end.checked_sub(run.source_range.start) else {
        return 0.0;
    };
    if relative_end > run.source_text.len()
        || !run.source_text.is_char_boundary(relative_start)
        || !run.source_text.is_char_boundary(relative_end)
    {
        return 0.0;
    }
    let source_start = byte_start.max(run.source_range.start);
    let source_end = byte_end.min(run.source_range.end);
    if source_start >= source_end {
        return 0.0;
    }
    debug_assert!(
        include_kerning || !run.include_kerning,
        "include_kerning=false requires an unkerned shaped run"
    );

    run.lines
        .iter()
        .map(|line| {
            measured_source_width_from_glyphs(
                &run.source_text,
                run.source_range.start,
                line.glyphs.as_slice(),
                source_start,
                source_end,
            )
        })
        .fold(0.0_f32, f32::max)
}

pub(crate) fn measured_grapheme_widths_from_shaped(
    shaped: &ShapedGlyphRun,
    text: &str,
) -> Result<Vec<f32>, TextLayoutError> {
    project_grapheme_geometry_from_shaped(shaped, text, false).map(|geometry| geometry.advances)
}

pub(crate) fn measured_grapheme_geometry_from_shaped(
    shaped: &ShapedGlyphRun,
    text: &str,
) -> Result<MeasuredGraphemeGeometry, TextLayoutError> {
    project_grapheme_geometry_from_shaped(shaped, text, true)
}

fn project_grapheme_geometry_from_shaped(
    shaped: &ShapedGlyphRun,
    text: &str,
    retain_clusters: bool,
) -> Result<MeasuredGraphemeGeometry, TextLayoutError> {
    project_grapheme_geometry_with_tab_presence_from_shaped::<false>(shaped, text, retain_clusters)
        .map(|(geometry, _)| geometry)
}

fn project_grapheme_geometry_with_tab_presence_from_shaped<const TRACK_TAB: bool>(
    shaped: &ShapedGlyphRun,
    text: &str,
    retain_clusters: bool,
) -> Result<(MeasuredGraphemeGeometry, bool), TextLayoutError> {
    let source_offset = shaped.source_range.start;
    let mut contains_tab = false;
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| {
            if TRACK_TAB {
                contains_tab |= grapheme == "\t";
            }
            let end = start + grapheme.len();
            (source_offset + start, source_offset + end)
        })
        .collect::<Vec<_>>();
    let geometry = project_shaped_geometry_to_source_ranges(
        shaped,
        text,
        source_offset,
        source_offset,
        &graphemes,
        retain_clusters,
    )?;
    Ok((geometry, contains_tab))
}

/// Projects one shaped run onto caller-owned paragraph grapheme ranges.
///
/// `source_text_offset` maps byte zero of `source_text` into the absolute source coordinate space.
/// `shaped_source_offset` maps `shaped.source_range.start` into that same space. This lets a style
/// span retain its own shaping request while a grapheme that crosses the span boundary remains one
/// layout/caret unit owned by the paragraph.
pub(crate) fn project_shaped_geometry_to_source_ranges(
    shaped: &ShapedGlyphRun,
    source_text: &str,
    source_text_offset: usize,
    shaped_source_offset: usize,
    graphemes: &[(usize, usize)],
    retain_clusters: bool,
) -> Result<MeasuredGraphemeGeometry, TextLayoutError> {
    crate::profile_scope!("runtime", "text.measure", "grapheme_projection");
    validate_shaped_geometry_source(shaped, shaped.source_text.as_ref())?;
    let source_end = source_text_offset
        .checked_add(source_text.len())
        .ok_or(TextLayoutError::LayoutFailed)?;
    let shaped_delta = shaped_source_offset
        .checked_sub(shaped.source_range.start)
        .ok_or(TextLayoutError::LayoutFailed)?;
    let mut previous_end = source_text_offset;
    for &(start, end) in graphemes {
        if start < previous_end
            || start >= end
            || start < source_text_offset
            || end > source_end
            || source_text
                .get(start - source_text_offset..end - source_text_offset)
                .is_none()
        {
            return Err(TextLayoutError::LayoutFailed);
        }
        previous_end = end;
    }
    let mut widths = vec![0.0; graphemes.len()];
    let mut glyph_clusters = Vec::new();
    let mut projection_cursor = GraphemeProjectionCursor::default();

    // The same zero-allocation cluster iterator feeds measurement and renderer artifact geometry.
    // This keeps one backend cluster definition even when several glyphs cover one ligature.
    for line in &shaped.lines {
        for cluster in text_glyph_clusters(&line.glyphs) {
            let absolute_cluster = crate::text::TextRange {
                start: cluster
                    .source_range
                    .start
                    .checked_add(shaped_delta)
                    .ok_or(TextLayoutError::LayoutFailed)?,
                end: cluster
                    .source_range
                    .end
                    .checked_add(shaped_delta)
                    .ok_or(TextLayoutError::LayoutFailed)?,
            };
            let (first_overlapping, after_last_overlapping) = projection_cursor.overlap_bounds(
                graphemes,
                absolute_cluster.start,
                absolute_cluster.end,
            );
            if first_overlapping >= after_last_overlapping {
                continue;
            }

            let cluster_span = if let Some(cluster_span) = aligned_grapheme_span(
                graphemes,
                first_overlapping,
                after_last_overlapping,
                absolute_cluster,
            ) {
                cluster_span
            } else {
                source_grapheme_span(source_text, source_text_offset, absolute_cluster)
            };
            for index in first_overlapping..after_last_overlapping {
                let (source_start, source_end) = graphemes[index];
                widths[index] = finite_sum([
                    widths[index],
                    measured_source_range_overlap_with_span(
                        source_text,
                        source_text_offset,
                        absolute_cluster,
                        cluster.advance,
                        source_start,
                        source_end,
                        cluster_span,
                    ),
                ]);
            }
            if retain_clusters {
                let break_safety = line
                    .glyphs
                    .get(cluster.glyph_start)
                    .filter(|glyph| glyph.cluster_flags.cluster_start)
                    .map_or(ShapedGlyphBreakSafety::Unknown, |glyph| {
                        glyph.cluster_flags.break_safety
                    });
                glyph_clusters.push(MeasuredGlyphCluster {
                    source_range: absolute_cluster,
                    advance: cluster.advance,
                    caret_policy: if after_last_overlapping - first_overlapping > 1 {
                        MeasuredClusterCaretPolicy::AtomicCluster
                    } else {
                        MeasuredClusterCaretPolicy::GraphemeBoundary
                    },
                    break_safety,
                });
            }
        }
    }

    glyph_clusters.sort_by_key(|cluster| (cluster.source_range.start, cluster.source_range.end));
    Ok(MeasuredGraphemeGeometry {
        advances: widths,
        glyph_clusters,
    })
}

pub(crate) fn line_metrics_with_provider<P>(
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<TextLineMetrics>
where
    P: TextShapeRunProvider + ?Sized,
{
    let requested_line_height = resolved_line_height(style);
    shape_line_with_provider(DEFAULT_METRICS_SAMPLE, style, provider).map(|mut metrics| {
        metrics.line_height = requested_line_height.max(metrics.line_height);
        metrics.baseline = metrics.baseline.clamp(0.0, metrics.line_height);
        metrics
    })
}

fn shape_line_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<TextLineMetrics>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_unconstrained_line_with_provider(text, style, provider)
        .map(|shaped| text_line_metrics_from_shaped(&shaped, style))
}

pub(crate) fn text_line_metrics_from_shaped(
    shaped: &ShapedGlyphRun,
    style: &TextStyle,
) -> TextLineMetrics {
    shaped.lines.first().map_or(
        TextLineMetrics {
            width: 0.0,
            baseline: style.font_size.max(1.0) * 0.8,
            line_height: resolved_line_height(style),
        },
        |line| TextLineMetrics {
            width: line.measured_width,
            baseline: line.baseline,
            line_height: line.line_height,
        },
    )
}

#[cfg(test)]
fn shape_unconstrained_line(text: &str, style: &TextStyle) -> ShapedGlyphRun {
    shape_unconstrained_line_with_kerning(text, style, true)
}

pub(crate) fn shape_unconstrained_line_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<Arc<ShapedGlyphRun>>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_unconstrained_line_with_kerning_and_provider(text, style, true, provider)
}

#[cfg(test)]
fn shape_unconstrained_line_with_kerning(
    text: &str,
    style: &TextStyle,
    include_kerning: bool,
) -> ShapedGlyphRun {
    let mut provider = DirectTextShapeRunProvider::default();
    shape_unconstrained_line_with_kerning_and_provider(text, style, include_kerning, &mut provider)
        .into_result()
        .expect("test shaping request must be valid")
        .as_ref()
        .clone()
}

fn shape_unconstrained_line_with_kerning_and_provider<P>(
    text: &str,
    style: &TextStyle,
    include_kerning: bool,
    provider: &mut P,
) -> TextLayoutOutcome<Arc<ShapedGlyphRun>>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_horizontal_range_with_kerning_and_provider(
        text,
        style,
        TextDirection::Auto,
        TextRange {
            start: 0,
            end: text.len(),
        },
        include_kerning,
        provider,
    )
}

#[cfg(test)]
fn shape_horizontal_range(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
) -> ShapedGlyphRun {
    shape_horizontal_range_with_kerning(text, style, direction, source_range, true)
}

#[cfg(test)]
fn shape_horizontal_range_with_kerning(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
) -> ShapedGlyphRun {
    let mut provider = DirectTextShapeRunProvider::default();
    shape_horizontal_range_with_kerning_and_provider(
        text,
        style,
        direction,
        source_range,
        include_kerning,
        &mut provider,
    )
    .into_result()
    .expect("test shaping request must be valid")
    .as_ref()
    .clone()
}

fn shape_horizontal_range_with_kerning_and_provider<P>(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
    provider: &mut P,
) -> TextLayoutOutcome<Arc<ShapedGlyphRun>>
where
    P: TextShapeRunProvider + ?Sized,
{
    provider.shape_horizontal_range_with_kerning(
        text,
        style,
        direction,
        source_range,
        include_kerning,
    )
}

fn measured_source_width_from_glyphs(
    source_text: &str,
    source_offset: usize,
    glyphs: &[ShapedGlyph],
    source_start: usize,
    source_end: usize,
) -> f32 {
    finite_sum(glyphs.iter().map(|glyph| {
        measured_glyph_source_overlap(source_text, source_offset, glyph, source_start, source_end)
    }))
}

fn measured_glyph_source_overlap(
    source_text: &str,
    source_offset: usize,
    glyph: &ShapedGlyph,
    source_start: usize,
    source_end: usize,
) -> f32 {
    let glyph_span = source_grapheme_span(source_text, source_offset, glyph.source_range);
    measured_glyph_source_overlap_with_span(
        source_text,
        source_offset,
        glyph,
        source_start,
        source_end,
        glyph_span,
    )
}

fn measured_glyph_source_overlap_with_span(
    source_text: &str,
    source_offset: usize,
    glyph: &ShapedGlyph,
    source_start: usize,
    source_end: usize,
    glyph_span: f32,
) -> f32 {
    measured_source_range_overlap_with_span(
        source_text,
        source_offset,
        glyph.source_range,
        glyph.advance,
        source_start,
        source_end,
        glyph_span,
    )
}

#[allow(clippy::too_many_arguments)]
fn measured_source_range_overlap_with_span(
    source_text: &str,
    source_offset: usize,
    range: TextRange,
    advance: f32,
    source_start: usize,
    source_end: usize,
    range_span: f32,
) -> f32 {
    let overlap_start = range.start.max(source_start);
    let overlap_end = range.end.min(source_end);
    if overlap_start >= overlap_end {
        return 0.0;
    }

    let advance = advance.max(0.0);
    if overlap_start == range.start && overlap_end == range.end {
        return advance;
    }

    let overlap_span = source_grapheme_span(
        source_text,
        source_offset,
        TextRange {
            start: overlap_start,
            end: overlap_end,
        },
    );
    if range_span <= 0.0 || overlap_span <= 0.0 {
        return advance;
    }
    advance * (overlap_span / range_span).clamp(0.0, 1.0)
}

fn source_grapheme_span(source_text: &str, source_offset: usize, range: TextRange) -> f32 {
    let Some(start) = range.start.checked_sub(source_offset) else {
        return 0.0;
    };
    let Some(end) = range.end.checked_sub(source_offset) else {
        return 0.0;
    };
    let Some(source) = source_text.get(start..end) else {
        return 0.0;
    };
    source.graphemes(true).count().max(1) as f32
}

fn resolved_line_height(style: &TextStyle) -> f32 {
    style.line_height.max(style.font_size.max(1.0))
}

#[cfg(test)]
#[path = "measure/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "measure/tests/measured_line_contract_tests.rs"]
mod measured_line_contract_tests;
