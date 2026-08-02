use crate::text::shaping::TextShapeRunProvider;
use crate::text::{LaidOutText, TextStyle};

use super::rich_advance_index::RichAdvanceIndex;
use super::{
    line_break_chunks_with_provider, line_metrics_with_provider, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider, RichTextLayoutSource,
};

mod materialize;
mod metrics;

pub(crate) use materialize::layout_rich_line_with_provider;
use materialize::{layout_rich_ranges_with_index, HorizontalRichLayoutIndex};
use metrics::inline_box_metrics;
pub(crate) use metrics::resolve_rich_run_style;

pub(crate) fn layout_rich_text_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    provider: &mut P,
) -> LaidOutText
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let index = HorizontalRichLayoutIndex::new(source, style, provider);
    layout_rich_ranges_with_index(source, rich_forced_line_ranges(source.text()), &index)
}

pub(crate) fn layout_rich_text_glyph_wrapped_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    provider: &mut P,
) -> LaidOutText
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let index = HorizontalRichLayoutIndex::new(source, style, provider);
    let ranges = rich_glyph_line_ranges(source, max_width, &index.advances, provider);
    layout_rich_ranges_with_index(source, ranges, &index)
}

pub(crate) fn layout_rich_text_word_wrapped_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    provider: &mut P,
) -> (LaidOutText, Vec<(u32, u32)>)
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let index = HorizontalRichLayoutIndex::new(source, style, provider);
    let ranges = rich_word_line_ranges(source, style, max_width, mode, &index.advances, provider);
    let layout = layout_rich_ranges_with_index(source, ranges.clone(), &index);
    (layout, ranges)
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
) -> Vec<(u32, u32)>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let text_metrics = line_metrics_with_provider(style, provider);
    let text_ascent = text_metrics.baseline.max(0.0);
    let text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let advance_index = RichAdvanceIndex::new(source, style, provider, |inline, _| {
        let metrics = inline_box_metrics(inline, text_ascent, text_descent);
        (metrics.advance, metrics.size.y)
    });
    rich_glyph_line_ranges(source, max_width, &advance_index, provider)
}

fn rich_glyph_line_ranges<S, P>(
    source: &S,
    max_width: f32,
    advance_index: &RichAdvanceIndex,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let mut ranges = Vec::new();

    for forced_range in rich_forced_line_ranges(source.text()) {
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        let Some(text) = source.text().get(start..end) else {
            continue;
        };
        if text.is_empty() {
            ranges.push(forced_range);
        } else {
            ranges.extend(advance_index.corrected_glyph_ranges_with_provider(
                source.text(),
                start,
                end,
                max_width,
                max_width,
                provider,
            ));
        }
    }
    ranges
}

pub(crate) fn rich_word_line_ranges_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let text_metrics = line_metrics_with_provider(style, provider);
    let text_ascent = text_metrics.baseline.max(0.0);
    let text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let advance_index = RichAdvanceIndex::new(source, style, provider, |inline, _| {
        let metrics = inline_box_metrics(inline, text_ascent, text_descent);
        (metrics.advance, metrics.size.y)
    });
    rich_word_line_ranges(source, style, max_width, mode, &advance_index, provider)
}

fn rich_word_line_ranges<S, P>(
    source: &S,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    advance_index: &RichAdvanceIndex,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let mut ranges = Vec::new();

    for forced_range in rich_forced_line_ranges(source.text()) {
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        let Some(text) = source.text().get(start..end) else {
            continue;
        };
        let chunks = match mode {
            RichWordWrapMode::Word => line_break_chunks_with_provider(text, style, provider),
            RichWordWrapMode::WordSmart => {
                word_smart_line_break_chunks_with_provider(text, style, provider)
            }
        };
        if chunks.is_empty() {
            ranges.push(forced_range);
            continue;
        }

        let mut line_start = start;
        let mut line_end = start;
        for chunk in chunks {
            let mut chunk_start = start + chunk.source_range.start;
            let chunk_end = start + chunk.source_range.end;
            if line_end == line_start {
                chunk_start = trim_rich_leading_spaces(source.text(), chunk_start, chunk_end);
                line_start = chunk_start;
                line_end = chunk_start;
            }
            if chunk_start >= chunk_end {
                continue;
            }
            let break_suffix = chunk.break_suffix.map(|suffix| suffix.text);
            let mut candidate_width = advance_index.corrected_advance_with_provider(
                source.text(),
                line_start,
                chunk_end,
                break_suffix,
                provider,
            );
            if line_end > line_start && candidate_width > max_width {
                ranges.push((
                    u32::try_from(line_start).unwrap_or(u32::MAX),
                    u32::try_from(line_end).unwrap_or(u32::MAX),
                ));
                chunk_start = trim_rich_leading_spaces(source.text(), chunk_start, chunk_end);
                line_start = chunk_start;
                line_end = chunk_start;
                candidate_width = advance_index.corrected_advance_with_provider(
                    source.text(),
                    line_start,
                    chunk_end,
                    break_suffix,
                    provider,
                );
            }
            if chunk_start >= chunk_end {
                continue;
            }

            if line_end == line_start && candidate_width > max_width && chunk.allow_glyph_fallback {
                let fallback_ranges = advance_index.corrected_glyph_ranges_with_provider(
                    source.text(),
                    chunk_start,
                    chunk_end,
                    max_width,
                    max_width,
                    provider,
                );
                let fallback_count = fallback_ranges.len();
                for (index, range) in fallback_ranges.into_iter().enumerate() {
                    if index + 1 == fallback_count {
                        line_start = usize::try_from(range.0).unwrap_or(chunk_start);
                        line_end = usize::try_from(range.1).unwrap_or(chunk_end);
                    } else {
                        ranges.push(range);
                    }
                }
                continue;
            }
            line_end = chunk_end;
        }
        if line_end > line_start {
            ranges.push((
                u32::try_from(line_start).unwrap_or(u32::MAX),
                u32::try_from(line_end).unwrap_or(u32::MAX),
            ));
        }
    }
    ranges
}

fn trim_rich_leading_spaces(text: &str, start: usize, end: usize) -> usize {
    text.get(start..end)
        .map(|candidate| trim_leading_wrap_spaces(candidate, start).1)
        .unwrap_or(start)
}

pub(crate) fn rich_forced_line_ranges(text: &str) -> Vec<(u32, u32)> {
    crate::text::hard_lines(text)
        .into_iter()
        .map(|line| {
            (
                u32::try_from(line.content.start).unwrap_or(u32::MAX),
                u32::try_from(line.content.end).unwrap_or(u32::MAX),
            )
        })
        .collect()
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
