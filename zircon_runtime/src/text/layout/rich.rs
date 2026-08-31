use crate::core::framework::text::TextLayoutError;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{LaidOutText, TextStyle};

use super::rich_advance_index::RichAdvanceIndex;
use super::{
    RichTextLayoutSource, line_break_chunks_with_provider, line_metrics_with_provider,
    trim_leading_wrap_spaces, word_smart_line_break_chunks_with_provider,
};

mod materialize;
mod metrics;

pub(crate) use materialize::layout_rich_line_with_provider;
use materialize::{HorizontalRichLayoutIndex, layout_rich_ranges_with_index};
use metrics::inline_box_metrics;
pub(crate) use metrics::resolve_rich_run_style;

pub(crate) fn layout_rich_text_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<LaidOutText>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    HorizontalRichLayoutIndex::new(source, style, provider).and_then(|index| {
        rich_forced_line_ranges(source.text())
            .map(|ranges| layout_rich_ranges_with_index(source, ranges, &index))
    })
}

pub(crate) fn layout_rich_text_glyph_wrapped_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    provider: &mut P,
) -> TextLayoutOutcome<LaidOutText>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    HorizontalRichLayoutIndex::new(source, style, provider).and_then(|index| {
        rich_glyph_line_ranges(source, max_width, &index.advances, provider)
            .map(|ranges| layout_rich_ranges_with_index(source, ranges, &index))
    })
}

pub(crate) fn layout_rich_text_word_wrapped_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    provider: &mut P,
) -> TextLayoutOutcome<(LaidOutText, Vec<(u32, u32)>)>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    HorizontalRichLayoutIndex::new(source, style, provider).and_then(|index| {
        rich_word_line_ranges(source, style, max_width, mode, &index.advances, provider).map(
            |ranges| {
                let layout = layout_rich_ranges_with_index(source, ranges.clone(), &index);
                (layout, ranges)
            },
        )
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RichWordWrapMode {
    Word,
    WordSmart,
}

pub(crate) fn rich_glyph_line_ranges_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<(u32, u32)>>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let text_metrics = match line_metrics_with_provider(style, provider) {
        TextShapingOutcome::Ready(metrics) => metrics,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let text_ascent = text_metrics.baseline.max(0.0);
    let text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let advance_index = match RichAdvanceIndex::new(source, style, provider, |inline, _| {
        let metrics = inline_box_metrics(inline, text_ascent, text_descent);
        (metrics.advance, metrics.size.y)
    }) {
        TextShapingOutcome::Ready(index) => index,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    rich_glyph_line_ranges(source, max_width, &advance_index, provider)
}

fn rich_glyph_line_ranges<S, P>(
    source: &S,
    max_width: f32,
    advance_index: &RichAdvanceIndex,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<(u32, u32)>>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let mut ranges = Vec::new();

    let forced_ranges = match rich_forced_line_ranges(source.text()) {
        TextShapingOutcome::Ready(ranges) => ranges,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    for forced_range in forced_ranges {
        let (start, end) = match super::checked_source_range(source.text(), forced_range) {
            Ok(range) => range,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let Some(text) = source.text().get(start..end) else {
            return TextShapingOutcome::failed(
                crate::core::framework::text::TextLayoutError::LayoutFailed,
            );
        };
        if text.is_empty() {
            ranges.push(forced_range);
        } else {
            let corrected = match advance_index.corrected_glyph_ranges_with_provider(
                source.text(),
                start,
                end,
                max_width,
                max_width,
                provider,
            ) {
                TextShapingOutcome::Ready(ranges) => ranges,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            ranges.extend(corrected);
        }
    }
    TextShapingOutcome::Ready(ranges)
}

pub(crate) fn rich_word_line_ranges_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<(u32, u32)>>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let text_metrics = match line_metrics_with_provider(style, provider) {
        TextShapingOutcome::Ready(metrics) => metrics,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let text_ascent = text_metrics.baseline.max(0.0);
    let text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let advance_index = match RichAdvanceIndex::new(source, style, provider, |inline, _| {
        let metrics = inline_box_metrics(inline, text_ascent, text_descent);
        (metrics.advance, metrics.size.y)
    }) {
        TextShapingOutcome::Ready(index) => index,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    rich_word_line_ranges(source, style, max_width, mode, &advance_index, provider)
}

fn rich_word_line_ranges<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    advance_index: &RichAdvanceIndex,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<(u32, u32)>>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let mut ranges = Vec::new();

    let forced_ranges = match rich_forced_line_ranges(source.text()) {
        TextShapingOutcome::Ready(ranges) => ranges,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    for forced_range in forced_ranges {
        let (start, end) = match super::checked_source_range(source.text(), forced_range) {
            Ok(range) => range,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let Some(text) = source.text().get(start..end) else {
            return TextShapingOutcome::failed(
                crate::core::framework::text::TextLayoutError::LayoutFailed,
            );
        };
        let chunks = match match mode {
            RichWordWrapMode::Word => line_break_chunks_with_provider(text, style, provider),
            RichWordWrapMode::WordSmart => {
                word_smart_line_break_chunks_with_provider(text, style, provider)
            }
        } {
            TextShapingOutcome::Ready(chunks) => chunks,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        if chunks.is_empty() {
            ranges.push(forced_range);
            continue;
        }

        let mut line_start = start;
        let mut line_end = start;
        for chunk in chunks {
            let Some(mut chunk_start) = start.checked_add(chunk.source_range.start) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let Some(chunk_end) = start.checked_add(chunk.source_range.end) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            if chunk_start > chunk_end || chunk_end > end {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            }
            if line_end == line_start {
                chunk_start = match trim_rich_leading_spaces(source.text(), chunk_start, chunk_end)
                {
                    Some(start) => start,
                    None => return TextShapingOutcome::failed(TextLayoutError::LayoutFailed),
                };
                line_start = chunk_start;
                line_end = chunk_start;
            }
            if chunk_start >= chunk_end {
                continue;
            }
            let break_suffix = chunk.break_suffix.map(|suffix| suffix.marker_text());
            let mut candidate_width = match advance_index.corrected_advance_with_provider(
                source.text(),
                line_start,
                chunk_end,
                break_suffix,
                provider,
            ) {
                TextShapingOutcome::Ready(width) => width,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            if line_end > line_start && candidate_width > max_width {
                ranges.push(
                    match super::checked_source_range_to_u32(source.text(), line_start, line_end) {
                        Ok(range) => range,
                        Err(error) => return TextShapingOutcome::failed(error),
                    },
                );
                chunk_start = match trim_rich_leading_spaces(source.text(), chunk_start, chunk_end)
                {
                    Some(start) => start,
                    None => return TextShapingOutcome::failed(TextLayoutError::LayoutFailed),
                };
                line_start = chunk_start;
                line_end = chunk_start;
                candidate_width = match advance_index.corrected_advance_with_provider(
                    source.text(),
                    line_start,
                    chunk_end,
                    break_suffix,
                    provider,
                ) {
                    TextShapingOutcome::Ready(width) => width,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
            }
            if chunk_start >= chunk_end {
                continue;
            }

            if line_end == line_start && candidate_width > max_width && chunk.allow_glyph_fallback {
                let fallback_ranges = match advance_index.corrected_glyph_ranges_with_provider(
                    source.text(),
                    chunk_start,
                    chunk_end,
                    max_width,
                    max_width,
                    provider,
                ) {
                    TextShapingOutcome::Ready(ranges) => ranges,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                let fallback_count = fallback_ranges.len();
                for (index, range) in fallback_ranges.into_iter().enumerate() {
                    if index + 1 == fallback_count {
                        (line_start, line_end) =
                            match super::checked_source_range(source.text(), range) {
                                Ok(range) => range,
                                Err(error) => return TextShapingOutcome::failed(error),
                            };
                    } else {
                        ranges.push(range);
                    }
                }
                continue;
            }
            line_end = chunk_end;
        }
        if line_end > line_start {
            ranges.push(
                match super::checked_source_range_to_u32(source.text(), line_start, line_end) {
                    Ok(range) => range,
                    Err(error) => return TextShapingOutcome::failed(error),
                },
            );
        }
    }
    TextShapingOutcome::Ready(ranges)
}

fn trim_rich_leading_spaces(text: &str, start: usize, end: usize) -> Option<usize> {
    text.get(start..end)
        .map(|candidate| trim_leading_wrap_spaces(candidate, start).1)
}

pub(crate) fn rich_forced_line_ranges(text: &str) -> TextLayoutOutcome<Vec<(u32, u32)>> {
    let mut ranges = Vec::with_capacity(crate::text::hard_line_count(text));
    let mut conversion_error = None;
    crate::text::visit_hard_lines(text, |line| {
        if conversion_error.is_some() {
            return;
        }
        match super::checked_source_range_to_u32(text, line.content.start, line.content.end) {
            Ok(range) => ranges.push(range),
            Err(error) => {
                conversion_error = Some(error);
            }
        }
    });
    match conversion_error {
        Some(error) => TextShapingOutcome::failed(error),
        None => TextShapingOutcome::Ready(ranges),
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
mod tests;
