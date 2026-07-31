use crate::core::framework::text::TextDirection;
use crate::text::shaping::{resolve_bidi_base_direction, TextShapeRunProvider};
use crate::text::{InlineObjectRef, RichParseResult, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use super::advance_index::{GraphemeAdvanceIndex, GraphemeAdvanceMetric};
use super::line_break::{corrected_index_advance_with_provider, corrected_metric_ranges};
use super::{measured_grapheme_widths_with_provider, resolve_rich_run_style};

#[derive(Clone, Debug, Default)]
pub(crate) struct RichAdvanceIndex {
    index: GraphemeAdvanceIndex,
    text_spans: Vec<RichTextSpan>,
}

impl RichAdvanceIndex {
    pub(crate) fn new<P, F>(
        parsed: &RichParseResult,
        base_style: &TextStyle,
        provider: &mut P,
        mut inline_metrics: F,
    ) -> Self
    where
        P: TextShapeRunProvider + ?Sized,
        F: FnMut(&InlineObjectRef, &TextStyle) -> (f32, f32),
    {
        let spans = source_spans(parsed, base_style);
        let mut metrics = Vec::new();
        let mut text_spans = Vec::new();
        for span in spans {
            if let Some(inline) = span.inline {
                append_inline_metrics(
                    &mut metrics,
                    &parsed.text,
                    span.start,
                    span.end,
                    inline_metrics(inline, &span.style),
                );
            } else {
                append_text_metrics(
                    &mut metrics,
                    &parsed.text,
                    span.start,
                    span.end,
                    &span.style,
                    provider,
                );
                text_spans.push(RichTextSpan {
                    start: span.start,
                    end: span.end,
                    style: span.style,
                });
            }
        }

        Self {
            index: GraphemeAdvanceIndex::from_metrics(metrics),
            text_spans,
        }
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
    ) -> f32
    where
        P: TextShapeRunProvider + ?Sized,
    {
        let raw_advance = self.index.advance(start, end);
        let first = self.text_spans.partition_point(|span| span.end <= start);
        let after_last = self.text_spans.partition_point(|span| span.start < end);
        let Some(last) = after_last.checked_sub(1) else {
            return raw_advance;
        };
        if first > last || self.text_spans.get(first).is_none() {
            return raw_advance;
        }

        if first == last {
            let span = &self.text_spans[first];
            if start <= span.start && end >= span.end && break_suffix.is_none() {
                return raw_advance;
            }
            let span_start = start.max(span.start);
            let span_end = end.min(span.end);
            return raw_advance - self.index.advance(span_start, span_end)
                + self.corrected_span_advance(
                    source,
                    span,
                    span_start,
                    span_end,
                    break_suffix,
                    provider,
                );
        }

        let mut corrected = raw_advance;
        let first_span = &self.text_spans[first];
        if start > first_span.start {
            let span_end = end.min(first_span.end);
            corrected -= self.index.advance(start, span_end);
            corrected +=
                self.corrected_span_advance(source, first_span, start, span_end, None, provider);
        }
        let last_span = &self.text_spans[last];
        if end < last_span.end || break_suffix.is_some() {
            let span_start = start.max(last_span.start);
            corrected -= self.index.advance(span_start, end);
            corrected += self.corrected_span_advance(
                source,
                last_span,
                span_start,
                end,
                break_suffix,
                provider,
            );
        }
        finite_non_negative(corrected)
    }

    pub(crate) fn corrected_glyph_ranges_with_provider<P>(
        &self,
        source: &str,
        start: usize,
        end: usize,
        first_max_advance: f32,
        continuation_max_advance: f32,
        provider: &mut P,
    ) -> Vec<(u32, u32)>
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
        .into_iter()
        .map(|(first, after_last)| {
            (
                to_u32(metrics[first].source_start),
                to_u32(metrics[after_last.saturating_sub(1)].source_end),
            )
        })
        .collect()
    }

    fn corrected_span_advance<P>(
        &self,
        source: &str,
        span: &RichTextSpan,
        start: usize,
        end: usize,
        break_suffix: Option<&str>,
        provider: &mut P,
    ) -> f32
    where
        P: TextShapeRunProvider + ?Sized,
    {
        let direction = source
            .get(span.start..span.end)
            .map_or(TextDirection::Auto, |text| {
                resolve_bidi_base_direction(text, TextDirection::Auto)
            });
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
struct RichTextSpan {
    start: usize,
    end: usize,
    style: TextStyle,
}

struct SourceSpan<'a> {
    start: usize,
    end: usize,
    style: TextStyle,
    inline: Option<&'a InlineObjectRef>,
}

fn source_spans<'a>(parsed: &'a RichParseResult, base_style: &TextStyle) -> Vec<SourceSpan<'a>> {
    let mut spans = Vec::with_capacity(parsed.runs.len().saturating_add(1));
    let mut cursor = 0;
    for run in &parsed.runs {
        let Some((run_start, run_end)) = valid_source_range(run.byte_range, parsed.text.len())
        else {
            continue;
        };
        if run_end <= cursor {
            continue;
        }
        if run_start > cursor {
            push_text_span(&mut spans, cursor, run_start, base_style.clone());
        }
        let start = run_start.max(cursor);
        let style = resolve_rich_run_style(base_style, &run.style);
        if let Some(inline) = run.inline.as_ref() {
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
    }
    if cursor < parsed.text.len() {
        push_text_span(&mut spans, cursor, parsed.text.len(), base_style.clone());
    }
    spans
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
    source: &str,
    start: usize,
    end: usize,
    style: &TextStyle,
    provider: &mut P,
) where
    P: TextShapeRunProvider + ?Sized,
{
    let Some(text) = source.get(start..end) else {
        return;
    };
    let cross_extent = finite_non_negative(style.font_size.max(1.0));
    let mut segment_start = 0;
    for (newline_start, newline) in text.match_indices('\n') {
        append_shaped_segment(
            metrics,
            text,
            start,
            segment_start,
            newline_start,
            cross_extent,
            style,
            provider,
        );
        let source_start = start + newline_start;
        metrics.push(GraphemeAdvanceMetric {
            source_start,
            source_end: source_start + newline.len(),
            advance: 0.0,
            cross_extent,
        });
        segment_start = newline_start + newline.len();
    }
    append_shaped_segment(
        metrics,
        text,
        start,
        segment_start,
        text.len(),
        cross_extent,
        style,
        provider,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_shaped_segment<P>(
    metrics: &mut Vec<GraphemeAdvanceMetric>,
    span_text: &str,
    span_source_start: usize,
    segment_start: usize,
    segment_end: usize,
    cross_extent: f32,
    style: &TextStyle,
    provider: &mut P,
) where
    P: TextShapeRunProvider + ?Sized,
{
    let Some(text) = span_text.get(segment_start..segment_end) else {
        return;
    };
    if text.is_empty() {
        return;
    }
    let advances = measured_grapheme_widths_with_provider(text, style, provider);
    for (index, (offset, grapheme)) in text.grapheme_indices(true).enumerate() {
        let source_start = span_source_start + segment_start + offset;
        metrics.push(GraphemeAdvanceMetric {
            source_start,
            source_end: source_start + grapheme.len(),
            advance: advances
                .get(index)
                .copied()
                .map_or(0.0, finite_non_negative),
            cross_extent,
        });
    }
}

fn append_inline_metrics(
    metrics: &mut Vec<GraphemeAdvanceMetric>,
    source: &str,
    start: usize,
    end: usize,
    inline_metrics: (f32, f32),
) {
    let Some(text) = source.get(start..end) else {
        return;
    };
    let advance = finite_non_negative(inline_metrics.0);
    let cross_extent = finite_non_negative(inline_metrics.1);
    for (index, (offset, grapheme)) in text.grapheme_indices(true).enumerate() {
        let source_start = start + offset;
        metrics.push(GraphemeAdvanceMetric {
            source_start,
            source_end: source_start + grapheme.len(),
            advance: if index == 0 { advance } else { 0.0 },
            cross_extent,
        });
    }
}

fn valid_source_range(range: (u32, u32), source_len: usize) -> Option<(usize, usize)> {
    let start = usize::try_from(range.0).ok()?;
    let end = usize::try_from(range.1).ok()?.min(source_len);
    (start < end && start <= source_len).then_some((start, end))
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
