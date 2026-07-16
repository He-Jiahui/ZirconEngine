use crate::text::shaping::TextShapeRunProvider;
use crate::text::{InlineObjectRef, RichParseResult};
use crate::text::{TextStyle, TextWrap};
use unicode_segmentation::UnicodeSegmentation;

use super::{
    line_break_chunks_with_provider, measured_grapheme_widths_with_provider,
    resolve_rich_run_style, rich_forced_line_ranges, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RichVerticalColumnMetrics {
    pub(crate) source_range: (u32, u32),
    pub(crate) advances: Vec<f32>,
    pub(crate) cross_extent: f32,
}

pub(crate) fn rich_vertical_columns_with_provider<P, F>(
    parsed: &RichParseResult,
    style: &TextStyle,
    mut max_height_for_column: F,
    provider: &mut P,
) -> Vec<RichVerticalColumnMetrics>
where
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let ranges = match style.wrap {
        TextWrap::None => rich_forced_line_ranges(&parsed.text),
        TextWrap::Glyph => glyph_column_ranges(parsed, style, &mut max_height_for_column, provider),
        TextWrap::Word | TextWrap::WordSmart => {
            word_column_ranges(parsed, style, &mut max_height_for_column, provider)
        }
    };

    ranges
        .into_iter()
        .map(|source_range| {
            let (advances, cross_extent) =
                source_range_metrics(parsed, style, source_range, provider);
            RichVerticalColumnMetrics {
                source_range,
                advances,
                cross_extent,
            }
        })
        .collect()
}

fn glyph_column_ranges<P, F>(
    parsed: &RichParseResult,
    style: &TextStyle,
    max_height_for_column: &mut F,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let mut ranges = Vec::new();
    for forced_range in rich_forced_line_ranges(&parsed.text) {
        let first_max_height = finite_non_negative(max_height_for_column(forced_range, 0));
        let continuation_max_height = finite_non_negative(max_height_for_column(forced_range, 1));
        ranges.extend(glyph_ranges_for_source_range(
            parsed,
            style,
            forced_range,
            first_max_height,
            continuation_max_height,
            provider,
        ));
    }
    ranges
}

fn word_column_ranges<P, F>(
    parsed: &RichParseResult,
    style: &TextStyle,
    max_height_for_column: &mut F,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let mut ranges = Vec::new();
    for forced_range in rich_forced_line_ranges(&parsed.text) {
        let first_max_height = finite_non_negative(max_height_for_column(forced_range, 0));
        let continuation_max_height = finite_non_negative(max_height_for_column(forced_range, 1));
        let mut max_height = first_max_height;
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        let Some(text) = parsed.text.get(start..end) else {
            continue;
        };
        let chunks = if matches!(style.wrap, TextWrap::WordSmart) {
            word_smart_line_break_chunks_with_provider(text, style, provider)
        } else {
            line_break_chunks_with_provider(text, style, provider)
        };
        if chunks.is_empty() {
            ranges.push(forced_range);
            continue;
        }

        let mut column_start = start;
        let mut column_end = start;
        let mut column_height = 0.0_f32;
        for chunk in chunks {
            let mut chunk_start = start + chunk.source_range.start;
            let chunk_end = start + chunk.source_range.end;
            if column_end == column_start {
                chunk_start = trim_rich_leading_spaces(&parsed.text, chunk_start, chunk_end);
                column_start = chunk_start;
                column_end = chunk_start;
            }
            if chunk_start >= chunk_end {
                continue;
            }
            let chunk_height = source_range_advance(
                parsed,
                style,
                (to_u32(chunk_start), to_u32(chunk_end)),
                provider,
            );
            if column_end > column_start && column_height + chunk_height > max_height {
                ranges.push((to_u32(column_start), to_u32(column_end)));
                chunk_start = trim_rich_leading_spaces(&parsed.text, chunk_start, chunk_end);
                column_start = chunk_start;
                column_end = chunk_start;
                column_height = 0.0;
                max_height = continuation_max_height;
            }
            if chunk_start >= chunk_end {
                continue;
            }

            let chunk_height = source_range_advance(
                parsed,
                style,
                (to_u32(chunk_start), to_u32(chunk_end)),
                provider,
            );
            if column_end == column_start && chunk_height > max_height && chunk.allow_glyph_fallback
            {
                let fallback = glyph_ranges_for_source_range(
                    parsed,
                    style,
                    (to_u32(chunk_start), to_u32(chunk_end)),
                    max_height,
                    continuation_max_height,
                    provider,
                );
                let fallback_count = fallback.len();
                for (index, range) in fallback.into_iter().enumerate() {
                    if index + 1 == fallback_count {
                        column_start = usize::try_from(range.0).unwrap_or(chunk_start);
                        column_end = usize::try_from(range.1).unwrap_or(chunk_end);
                        column_height = source_range_advance(parsed, style, range, provider);
                    } else {
                        ranges.push(range);
                    }
                }
                if fallback_count > 1 {
                    max_height = continuation_max_height;
                }
                continue;
            }
            column_end = chunk_end;
            column_height += chunk_height;
        }
        if column_end > column_start {
            ranges.push((to_u32(column_start), to_u32(column_end)));
        }
    }
    ranges
}

fn glyph_ranges_for_source_range<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    source_range: (u32, u32),
    first_max_height: f32,
    continuation_max_height: f32,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
{
    let start = usize::try_from(source_range.0).unwrap_or(usize::MAX);
    let end = usize::try_from(source_range.1).unwrap_or(usize::MAX);
    let Some(text) = parsed.text.get(start..end) else {
        return Vec::new();
    };
    if text.is_empty() {
        return vec![source_range];
    }

    let mut ranges = Vec::new();
    let mut column_start = start;
    let mut column_height = 0.0_f32;
    let mut max_height = first_max_height;
    for (offset, grapheme) in text.grapheme_indices(true) {
        let grapheme_start = start + offset;
        let grapheme_end = grapheme_start + grapheme.len();
        let advance = grapheme_metrics(parsed, style, grapheme, grapheme_start, provider).0;
        if grapheme_start > column_start && column_height + advance > max_height {
            ranges.push((to_u32(column_start), to_u32(grapheme_start)));
            column_start = grapheme_start;
            column_height = 0.0;
            max_height = continuation_max_height;
        }
        column_height += advance;
        if grapheme_end == end {
            ranges.push((to_u32(column_start), to_u32(end)));
        }
    }
    ranges
}

fn source_range_metrics<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    source_range: (u32, u32),
    provider: &mut P,
) -> (Vec<f32>, f32)
where
    P: TextShapeRunProvider + ?Sized,
{
    let start = usize::try_from(source_range.0).unwrap_or(usize::MAX);
    let end = usize::try_from(source_range.1).unwrap_or(usize::MAX);
    if parsed.text.get(start..end).is_none() {
        return (Vec::new(), style.font_size.max(1.0));
    }

    let mut advances = Vec::new();
    let mut cross_extent = style.font_size.max(1.0);
    for run in &parsed.runs {
        let run_start = usize::try_from(run.byte_range.0)
            .unwrap_or(usize::MAX)
            .max(start);
        let run_end = usize::try_from(run.byte_range.1)
            .unwrap_or_default()
            .min(end);
        if run_start >= run_end {
            continue;
        }
        let run_style = resolve_rich_run_style(style, &run.style);
        if let Some(inline) = run.inline.as_ref() {
            let size = inline_size(inline, &run_style);
            advances.push(size.1);
            cross_extent = cross_extent.max(size.0);
        } else if let Some(text) = parsed.text.get(run_start..run_end) {
            advances.extend(measured_grapheme_widths_with_provider(
                text, &run_style, provider,
            ));
            cross_extent = cross_extent.max(run_style.font_size.max(1.0));
        }
    }
    (advances, cross_extent)
}

fn source_range_advance<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    source_range: (u32, u32),
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    source_range_metrics(parsed, style, source_range, provider)
        .0
        .into_iter()
        .sum()
}

fn trim_rich_leading_spaces(text: &str, start: usize, end: usize) -> usize {
    text.get(start..end)
        .map(|candidate| trim_leading_wrap_spaces(candidate, start).1)
        .unwrap_or(start)
}

fn grapheme_metrics<P>(
    parsed: &RichParseResult,
    base_style: &TextStyle,
    grapheme: &str,
    source_start: usize,
    provider: &mut P,
) -> (f32, f32)
where
    P: TextShapeRunProvider + ?Sized,
{
    let run = parsed.runs.iter().find(|run| {
        usize::try_from(run.byte_range.0)
            .ok()
            .is_some_and(|start| start <= source_start)
            && usize::try_from(run.byte_range.1)
                .ok()
                .is_some_and(|end| source_start < end)
    });
    let style = run
        .map(|run| resolve_rich_run_style(base_style, &run.style))
        .unwrap_or_else(|| base_style.clone());
    if let Some(inline) = run.and_then(|run| run.inline.as_ref()) {
        let size = inline_size(inline, &style);
        return (size.1, size.0);
    }
    let advance = measured_grapheme_widths_with_provider(grapheme, &style, provider)
        .into_iter()
        .sum::<f32>();
    (advance, style.font_size.max(1.0))
}

fn inline_size(inline: &InlineObjectRef, style: &TextStyle) -> (f32, f32) {
    let size = match inline {
        InlineObjectRef::Image { size, .. } | InlineObjectRef::Widget { size, .. } => *size,
        InlineObjectRef::Icon { .. } => {
            let extent = style.font_size.max(1.0);
            crate::core::math::Vec2::new(extent, extent)
        }
    };
    (finite_non_negative(size.x), finite_non_negative(size.y))
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
