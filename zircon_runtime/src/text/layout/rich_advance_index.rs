use crate::core::framework::text::TextDirection;
use crate::core::framework::text::TextLayoutError;
use crate::text::layout_geometry::{finite_sum, FiniteGeometryAccumulator};
use crate::text::shaping::{
    resolve_bidi_base_direction, TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome,
};
use crate::text::{InlineObjectRef, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use super::advance_index::{GraphemeAdvanceIndex, GraphemeAdvanceMetric};
use super::line_break::{corrected_index_advance_with_provider, corrected_metric_ranges};
use super::measure::{
    project_shaped_geometry_to_source_ranges, shape_unconstrained_line_with_provider,
    MeasuredClusterCaretPolicy, MeasuredGlyphCluster,
};
use super::{resolve_rich_run_style, RichTextLayoutSource};

#[derive(Clone, Debug, Default)]
pub(crate) struct RichAdvanceIndex {
    index: GraphemeAdvanceIndex,
    text_spans: Vec<ResolvedRichTextSpan>,
}

impl RichAdvanceIndex {
    pub(crate) fn new<S, P, F>(
        source: &S,
        base_style: &TextStyle,
        provider: &mut P,
        mut inline_metrics: F,
    ) -> TextLayoutOutcome<Self>
    where
        S: RichTextLayoutSource + ?Sized,
        P: TextShapeRunProvider + ?Sized,
        F: FnMut(&InlineObjectRef, &TextStyle) -> (f32, f32),
    {
        let spans = match source_spans(source, base_style) {
            Ok(spans) => spans,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let paragraph_grapheme_ranges = source
            .text()
            .grapheme_indices(true)
            .map(|(start, grapheme)| (start, start.saturating_add(grapheme.len())))
            .collect::<Vec<_>>();
        let mut metrics = paragraph_grapheme_ranges
            .iter()
            .map(|&(source_start, source_end)| GraphemeAdvanceMetric {
                source_start,
                source_end,
                advance: 0.0,
                cross_extent: 0.0,
            })
            .collect::<Vec<_>>();
        let mut glyph_clusters = Vec::new();
        let mut text_spans = Vec::new();
        for span in spans {
            if let Some(inline) = span.inline {
                match append_inline_metrics(
                    &mut metrics,
                    &mut glyph_clusters,
                    source.text(),
                    span.start,
                    span.end,
                    &paragraph_grapheme_ranges,
                    inline_metrics(inline, &span.style),
                ) {
                    TextShapingOutcome::Ready(()) => {}
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                }
            } else {
                match append_text_metrics(
                    &mut metrics,
                    &mut glyph_clusters,
                    source.text(),
                    span.start,
                    span.end,
                    &paragraph_grapheme_ranges,
                    &span.style,
                    provider,
                ) {
                    TextShapingOutcome::Ready(()) => {}
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                }
                text_spans.push(ResolvedRichTextSpan {
                    start: span.start,
                    end: span.end,
                    style: span.style,
                });
            }
        }

        TextShapingOutcome::Ready(Self {
            index: GraphemeAdvanceIndex::from_metrics_and_clusters(metrics, glyph_clusters),
            text_spans,
        })
    }

    pub(crate) fn metrics_in_range(&self, start: usize, end: usize) -> &[GraphemeAdvanceMetric] {
        self.index.metrics_in_range(start, end)
    }

    pub(crate) fn advance(&self, start: usize, end: usize) -> f32 {
        self.index.advance(start, end)
    }

    pub(crate) fn advances_and_max_cross(
        &self,
        start: usize,
        end: usize,
        minimum_cross_extent: f32,
    ) -> (Vec<f32>, f32) {
        self.index
            .advances_and_max_cross(start, end, minimum_cross_extent)
    }

    pub(crate) fn corrected_advance_with_provider<P>(
        &self,
        source: &str,
        start: usize,
        end: usize,
        break_suffix: Option<&str>,
        provider: &mut P,
    ) -> TextLayoutOutcome<f32>
    where
        P: TextShapeRunProvider + ?Sized,
    {
        if start > end || source.get(start..end).is_none() {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        }
        let raw_advance = self.index.advance(start, end);
        let first = self.text_spans.partition_point(|span| span.end <= start);
        let after_last = self.text_spans.partition_point(|span| span.start < end);
        let Some(last) = after_last.checked_sub(1) else {
            return TextShapingOutcome::Ready(raw_advance);
        };
        if first > last || self.text_spans.get(first).is_none() {
            return TextShapingOutcome::Ready(raw_advance);
        }

        if first == last {
            let span = &self.text_spans[first];
            if start <= span.start && end >= span.end && break_suffix.is_none() {
                return TextShapingOutcome::Ready(raw_advance);
            }
            let span_start = start.max(span.start);
            let span_end = end.min(span.end);
            return self
                .corrected_span_advance(source, span, span_start, span_end, break_suffix, provider)
                .map(|corrected| {
                    let mut resolved = FiniteGeometryAccumulator::default();
                    resolved.add(raw_advance);
                    resolved.add(-self.index.advance(span_start, span_end));
                    resolved.add(corrected);
                    finite_non_negative(resolved.value())
                });
        }

        let mut corrected = FiniteGeometryAccumulator::default();
        corrected.add(raw_advance);
        let first_span = &self.text_spans[first];
        if start > first_span.start {
            let span_end = end.min(first_span.end);
            corrected.add(-self.index.advance(start, span_end));
            corrected.add(
                match self
                    .corrected_span_advance(source, first_span, start, span_end, None, provider)
                {
                    TextShapingOutcome::Ready(advance) => advance,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error)
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                },
            );
        }
        let last_span = &self.text_spans[last];
        if end < last_span.end || break_suffix.is_some() {
            let span_start = start.max(last_span.start);
            corrected.add(-self.index.advance(span_start, end));
            corrected.add(
                match self.corrected_span_advance(
                    source,
                    last_span,
                    span_start,
                    end,
                    break_suffix,
                    provider,
                ) {
                    TextShapingOutcome::Ready(advance) => advance,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error)
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                },
            );
        }
        TextShapingOutcome::Ready(finite_non_negative(corrected.value()))
    }

    pub(crate) fn corrected_glyph_ranges_with_provider<P>(
        &self,
        source: &str,
        start: usize,
        end: usize,
        first_max_advance: f32,
        continuation_max_advance: f32,
        provider: &mut P,
    ) -> TextLayoutOutcome<Vec<(u32, u32)>>
    where
        P: TextShapeRunProvider + ?Sized,
    {
        let metrics = self.index.metrics_in_range(start, end);
        corrected_metric_ranges(
            metrics,
            first_max_advance,
            continuation_max_advance,
            |first, after_last| {
                self.corrected_advance_with_provider(
                    source,
                    metrics[first].source_start,
                    metrics[after_last.saturating_sub(1)].source_end,
                    None,
                    provider,
                )
            },
        )
        .and_then(|ranges| {
            let ranges = ranges
                .into_iter()
                .map(|(first, after_last)| {
                    (
                        metrics[first].source_start,
                        metrics[after_last.saturating_sub(1)].source_end,
                    )
                })
                .collect::<Vec<_>>();
            let result = ranges
                .into_iter()
                .map(|(start, end)| {
                    Ok((
                        u32::try_from(start).map_err(|_| TextLayoutError::LayoutFailed)?,
                        u32::try_from(end).map_err(|_| TextLayoutError::LayoutFailed)?,
                    ))
                })
                .collect::<Result<Vec<_>, TextLayoutError>>();
            TextShapingOutcome::from_result(result)
        })
    }

    fn corrected_span_advance<P>(
        &self,
        source: &str,
        span: &ResolvedRichTextSpan,
        start: usize,
        end: usize,
        break_suffix: Option<&str>,
        provider: &mut P,
    ) -> TextLayoutOutcome<f32>
    where
        P: TextShapeRunProvider + ?Sized,
    {
        let Some(span_text) = source.get(span.start..span.end) else {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        };
        let direction = resolve_bidi_base_direction(span_text, TextDirection::Auto);
        corrected_index_advance_with_provider(
            source,
            &self.index,
            start,
            end,
            &span.style,
            direction,
            break_suffix,
            provider,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedRichTextSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) style: TextStyle,
}

struct SourceSpan<'a> {
    start: usize,
    end: usize,
    style: TextStyle,
    inline: Option<&'a InlineObjectRef>,
}

fn source_spans<'a, S>(
    source: &'a S,
    base_style: &TextStyle,
) -> Result<Vec<SourceSpan<'a>>, TextLayoutError>
where
    S: RichTextLayoutSource + ?Sized,
{
    let mut spans = Vec::with_capacity(source.run_count().saturating_add(1));
    let mut cursor = 0;
    super::rich_source::for_each_validated_rich_run(source, |run, run_start, run_end| {
        if run_start < cursor || run_end <= cursor {
            return Err(TextLayoutError::LayoutFailed);
        }
        if run_start > cursor {
            push_text_span(&mut spans, cursor, run_start, base_style.clone());
        }
        let start = run_start;
        let style = resolve_rich_run_style(base_style, run.style);
        if let Some(inline) = run.inline {
            spans.push(SourceSpan {
                start,
                end: run_end,
                style,
                inline: Some(inline),
            });
        } else {
            push_text_span(&mut spans, start, run_end, style);
        }
        cursor = run_end;
        Ok(())
    })?;
    if cursor < source.text().len() {
        push_text_span(&mut spans, cursor, source.text().len(), base_style.clone());
    }
    Ok(spans)
}

/// Projects the exact contiguous non-inline spans that rich layout sends to the shaper.
///
/// Adjacent parser runs that resolve to the same text style intentionally coalesce here, so
/// prewarm clients share the canonical source/key boundary with layout.
pub(crate) fn resolved_text_spans<S>(
    source: &S,
    base_style: &TextStyle,
) -> Result<Vec<ResolvedRichTextSpan>, TextLayoutError>
where
    S: RichTextLayoutSource + ?Sized,
{
    source_spans(source, base_style).map(|spans| {
        spans
            .into_iter()
            .filter_map(|span| {
                span.inline.is_none().then_some(ResolvedRichTextSpan {
                    start: span.start,
                    end: span.end,
                    style: span.style,
                })
            })
            .collect()
    })
}

fn push_text_span<'a>(spans: &mut Vec<SourceSpan<'a>>, start: usize, end: usize, style: TextStyle) {
    if start >= end {
        return;
    }
    if let Some(previous) = spans.last_mut() {
        if previous.inline.is_none() && previous.end == start && previous.style == style {
            previous.end = end;
            return;
        }
    }
    spans.push(SourceSpan {
        start,
        end,
        style,
        inline: None,
    });
}

fn append_text_metrics<P>(
    metrics: &mut Vec<GraphemeAdvanceMetric>,
    glyph_clusters: &mut Vec<MeasuredGlyphCluster>,
    source: &str,
    start: usize,
    end: usize,
    paragraph_graphemes: &[(usize, usize)],
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<()>
where
    P: TextShapeRunProvider + ?Sized,
{
    let Some(text) = source.get(start..end) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    let cross_extent = finite_non_negative(style.font_size.max(1.0));
    for line in crate::text::hard_lines(text) {
        match append_shaped_segment(
            metrics,
            glyph_clusters,
            source,
            text,
            start,
            line.content.start,
            line.content.end,
            cross_extent,
            style,
            paragraph_graphemes,
            provider,
        ) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
        if !line.separator.is_empty() {
            let separator_start = match start.checked_add(line.separator.start) {
                Some(value) => value,
                None => return TextShapingOutcome::failed(TextLayoutError::LayoutFailed),
            };
            let separator_end = match start.checked_add(line.separator.end) {
                Some(value) => value,
                None => return TextShapingOutcome::failed(TextLayoutError::LayoutFailed),
            };
            accumulate_cross_extent(
                metrics,
                paragraph_graphemes,
                separator_start,
                separator_end,
                cross_extent,
            );
            glyph_clusters.push(MeasuredGlyphCluster {
                source_range: crate::text::TextRange {
                    start: separator_start,
                    end: separator_end,
                },
                advance: 0.0,
                caret_policy: MeasuredClusterCaretPolicy::GraphemeBoundary,
                break_safety: crate::text::ShapedGlyphBreakSafety::Safe,
            });
        }
    }
    TextShapingOutcome::Ready(())
}

#[allow(clippy::too_many_arguments)]
fn append_shaped_segment<P>(
    metrics: &mut Vec<GraphemeAdvanceMetric>,
    glyph_clusters: &mut Vec<MeasuredGlyphCluster>,
    source: &str,
    span_text: &str,
    span_source_start: usize,
    segment_start: usize,
    segment_end: usize,
    cross_extent: f32,
    style: &TextStyle,
    paragraph_graphemes: &[(usize, usize)],
    provider: &mut P,
) -> TextLayoutOutcome<()>
where
    P: TextShapeRunProvider + ?Sized,
{
    let Some(text) = span_text.get(segment_start..segment_end) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    if text.is_empty() {
        return TextShapingOutcome::Ready(());
    }
    let shaped = match shape_unconstrained_line_with_provider(text, style, provider) {
        TextShapingOutcome::Ready(shaped) => shaped,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let Some(source_offset) = span_source_start.checked_add(segment_start) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    let first = paragraph_graphemes.partition_point(|&(_, end)| end <= source_offset);
    let after_last = paragraph_graphemes
        .partition_point(|&(start, _)| start < source_offset.saturating_add(text.len()));
    let geometry = match project_shaped_geometry_to_source_ranges(
        shaped.as_ref(),
        source,
        0,
        source_offset,
        paragraph_graphemes
            .get(first..after_last)
            .unwrap_or_default(),
        true,
    ) {
        Ok(geometry) => geometry,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    for (offset, geometry) in geometry.advances.into_iter().enumerate() {
        if let Some(metric) = metrics.get_mut(first.saturating_add(offset)) {
            metric.advance = finite_sum([metric.advance, finite_non_negative(geometry)]);
            metric.cross_extent = metric.cross_extent.max(cross_extent);
        }
    }
    for cluster in geometry.glyph_clusters {
        glyph_clusters.push(cluster);
    }
    TextShapingOutcome::Ready(())
}

fn append_inline_metrics(
    metrics: &mut Vec<GraphemeAdvanceMetric>,
    glyph_clusters: &mut Vec<MeasuredGlyphCluster>,
    source: &str,
    start: usize,
    end: usize,
    paragraph_graphemes: &[(usize, usize)],
    inline_metrics: (f32, f32),
) -> TextLayoutOutcome<()> {
    let Some(text) = source.get(start..end) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    let advance = finite_non_negative(inline_metrics.0);
    let cross_extent = finite_non_negative(inline_metrics.1);
    let first = paragraph_graphemes.partition_point(|&(_, grapheme_end)| grapheme_end <= start);
    let after_last =
        paragraph_graphemes.partition_point(|&(grapheme_start, _)| grapheme_start < end);
    let grapheme_count = after_last.saturating_sub(first);
    for index in first..after_last {
        if let Some(metric) = metrics.get_mut(index) {
            metric.advance =
                finite_sum([metric.advance, if index == first { advance } else { 0.0 }]);
            metric.cross_extent = metric.cross_extent.max(cross_extent);
        }
    }
    if grapheme_count > 0 {
        glyph_clusters.push(MeasuredGlyphCluster {
            source_range: crate::text::TextRange { start, end },
            advance,
            caret_policy: if grapheme_count > 1 {
                MeasuredClusterCaretPolicy::AtomicCluster
            } else {
                MeasuredClusterCaretPolicy::GraphemeBoundary
            },
            break_safety: crate::text::ShapedGlyphBreakSafety::Safe,
        });
    }
    TextShapingOutcome::Ready(())
}

fn accumulate_cross_extent(
    metrics: &mut [GraphemeAdvanceMetric],
    paragraph_graphemes: &[(usize, usize)],
    start: usize,
    end: usize,
    cross_extent: f32,
) {
    let first = paragraph_graphemes.partition_point(|&(_, grapheme_end)| grapheme_end <= start);
    let after_last =
        paragraph_graphemes.partition_point(|&(grapheme_start, _)| grapheme_start < end);
    for index in first..after_last {
        if let Some(metric) = metrics.get_mut(index) {
            metric.cross_extent = metric.cross_extent.max(cross_extent);
        }
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "rich_advance_index/tests/cases.rs"]
mod tests;
