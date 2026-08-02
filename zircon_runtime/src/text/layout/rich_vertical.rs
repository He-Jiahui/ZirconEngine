use crate::text::shaping::TextShapeRunProvider;
use crate::text::InlineObjectRef;
use crate::text::{TextStyle, TextWrap};

use super::rich_advance_index::RichAdvanceIndex;
use super::{
    line_break_chunks_with_provider, rich_forced_line_ranges, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider, RichTextLayoutSource,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RichVerticalColumnMetrics {
    pub(crate) source_range: (u32, u32),
    pub(crate) advances: Vec<f32>,
    pub(crate) cross_extent: f32,
}

pub(crate) fn rich_vertical_columns_with_provider<S, P, F>(
    source: &S,
    style: &TextStyle,
    mut max_height_for_column: F,
    provider: &mut P,
) -> Vec<RichVerticalColumnMetrics>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let advance_index = RichAdvanceIndex::new(source, style, provider, |inline, run_style| {
        let (cross_extent, advance) = inline_size(inline, run_style);
        (advance, cross_extent)
    });
    let ranges = match style.wrap {
        TextWrap::None => rich_forced_line_ranges(source.text()),
        TextWrap::Glyph => glyph_column_ranges(
            &advance_index,
            source.text(),
            &mut max_height_for_column,
            provider,
        ),
        TextWrap::Word | TextWrap::WordSmart => word_column_ranges(
            source,
            style,
            &advance_index,
            &mut max_height_for_column,
            provider,
        ),
    };

    ranges
        .into_iter()
        .map(|source_range| {
            let start = usize::try_from(source_range.0).unwrap_or(usize::MAX);
            let end = usize::try_from(source_range.1).unwrap_or(usize::MAX);
            let (advances, cross_extent) =
                advance_index.advances_and_max_cross(start, end, style.font_size.max(1.0));
            RichVerticalColumnMetrics {
                source_range,
                advances,
                cross_extent,
            }
        })
        .collect()
}

fn glyph_column_ranges<P, F>(
    advance_index: &RichAdvanceIndex,
    text: &str,
    max_height_for_column: &mut F,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let mut ranges = Vec::new();
    for forced_range in rich_forced_line_ranges(text) {
        let first_max_height = finite_non_negative(max_height_for_column(forced_range, 0));
        let continuation_max_height = finite_non_negative(max_height_for_column(forced_range, 1));
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        if start == end {
            ranges.push(forced_range);
        } else {
            ranges.extend(advance_index.corrected_glyph_ranges_with_provider(
                text,
                start,
                end,
                first_max_height,
                continuation_max_height,
                provider,
            ));
        }
    }
    ranges
}

fn word_column_ranges<S, P, F>(
    source: &S,
    style: &TextStyle,
    advance_index: &RichAdvanceIndex,
    max_height_for_column: &mut F,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let mut ranges = Vec::new();
    for forced_range in rich_forced_line_ranges(source.text()) {
        let first_max_height = finite_non_negative(max_height_for_column(forced_range, 0));
        let continuation_max_height = finite_non_negative(max_height_for_column(forced_range, 1));
        let mut max_height = first_max_height;
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        let Some(text) = source.text().get(start..end) else {
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
        for chunk in chunks {
            let mut chunk_start = start + chunk.source_range.start;
            let chunk_end = start + chunk.source_range.end;
            if column_end == column_start {
                chunk_start = trim_rich_leading_spaces(source.text(), chunk_start, chunk_end);
                column_start = chunk_start;
                column_end = chunk_start;
            }
            if chunk_start >= chunk_end {
                continue;
            }
            let break_suffix = chunk.break_suffix.map(|suffix| suffix.text);
            let mut candidate_height = advance_index.corrected_advance_with_provider(
                source.text(),
                column_start,
                chunk_end,
                break_suffix,
                provider,
            );
            if column_end > column_start && candidate_height > max_height {
                ranges.push((to_u32(column_start), to_u32(column_end)));
                chunk_start = trim_rich_leading_spaces(source.text(), chunk_start, chunk_end);
                column_start = chunk_start;
                column_end = chunk_start;
                max_height = continuation_max_height;
                candidate_height = advance_index.corrected_advance_with_provider(
                    source.text(),
                    column_start,
                    chunk_end,
                    break_suffix,
                    provider,
                );
            }
            if chunk_start >= chunk_end {
                continue;
            }

            if column_end == column_start
                && candidate_height > max_height
                && chunk.allow_glyph_fallback
            {
                let fallback = advance_index.corrected_glyph_ranges_with_provider(
                    source.text(),
                    chunk_start,
                    chunk_end,
                    max_height,
                    continuation_max_height,
                    provider,
                );
                let fallback_count = fallback.len();
                for (index, range) in fallback.into_iter().enumerate() {
                    if index + 1 == fallback_count {
                        column_start = usize::try_from(range.0).unwrap_or(chunk_start);
                        column_end = usize::try_from(range.1).unwrap_or(chunk_end);
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
        }
        if column_end > column_start {
            ranges.push((to_u32(column_start), to_u32(column_end)));
        }
    }
    ranges
}

fn trim_rich_leading_spaces(text: &str, start: usize, end: usize) -> usize {
    text.get(start..end)
        .map(|candidate| trim_leading_wrap_spaces(candidate, start).1)
        .unwrap_or(start)
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

#[cfg(test)]
mod tests;
