use crate::text::InlineObjectRef;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{TextStyle, TextWrap};

use super::rich_advance_index::RichAdvanceIndex;
use super::{
    RichTextLayoutSource, checked_source_range, checked_source_range_to_u32,
    line_break_chunks_with_provider, rich_forced_line_ranges, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider,
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
) -> TextLayoutOutcome<Vec<RichVerticalColumnMetrics>>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let advance_index = match RichAdvanceIndex::new(source, style, provider, |inline, run_style| {
        let (cross_extent, advance) = inline_size(inline, run_style);
        (advance, cross_extent)
    }) {
        TextShapingOutcome::Ready(index) => index,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let ranges = match style.wrap {
        TextWrap::None => match rich_forced_line_ranges(source.text()) {
            TextShapingOutcome::Ready(ranges) => ranges,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        },
        TextWrap::Glyph => match glyph_column_ranges(
            &advance_index,
            source.text(),
            &mut max_height_for_column,
            provider,
        ) {
            TextShapingOutcome::Ready(ranges) => ranges,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        },
        TextWrap::Word | TextWrap::WordSmart => match word_column_ranges(
            source,
            style,
            &advance_index,
            &mut max_height_for_column,
            provider,
        ) {
            TextShapingOutcome::Ready(ranges) => ranges,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        },
    };

    match ranges
        .into_iter()
        .map(|source_range| {
            let (start, end) = checked_source_range(source.text(), source_range)?;
            let (advances, cross_extent) =
                advance_index.advances_and_max_cross(start, end, style.font_size.max(1.0));
            Ok(RichVerticalColumnMetrics {
                source_range,
                advances,
                cross_extent,
            })
        })
        .collect::<Result<Vec<_>, crate::core::framework::text::TextLayoutError>>()
    {
        Ok(columns) => TextShapingOutcome::Ready(columns),
        Err(error) => TextShapingOutcome::failed(error),
    }
}

fn glyph_column_ranges<P, F>(
    advance_index: &RichAdvanceIndex,
    text: &str,
    max_height_for_column: &mut F,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<(u32, u32)>>
where
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let mut ranges = Vec::new();
    let forced_ranges = match rich_forced_line_ranges(text) {
        TextShapingOutcome::Ready(ranges) => ranges,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    for forced_range in forced_ranges {
        let (start, end) = match checked_source_range(text, forced_range) {
            Ok(range) => range,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let first_max_height = finite_non_negative(max_height_for_column(forced_range, 0));
        let continuation_max_height = finite_non_negative(max_height_for_column(forced_range, 1));
        if start == end {
            ranges.push(forced_range);
        } else {
            let corrected = match advance_index.corrected_glyph_ranges_with_provider(
                text,
                start,
                end,
                first_max_height,
                continuation_max_height,
                provider,
            ) {
                TextShapingOutcome::Ready(ranges) => ranges,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            for range in corrected {
                if let Err(error) = checked_source_range(text, range) {
                    return TextShapingOutcome::failed(error);
                }
                ranges.push(range);
            }
        }
    }
    TextShapingOutcome::Ready(ranges)
}

fn word_column_ranges<S, P, F>(
    source: &S,
    style: &TextStyle,
    advance_index: &RichAdvanceIndex,
    max_height_for_column: &mut F,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<(u32, u32)>>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
    F: FnMut((u32, u32), usize) -> f32,
{
    let mut ranges = Vec::new();
    let forced_ranges = match rich_forced_line_ranges(source.text()) {
        TextShapingOutcome::Ready(ranges) => ranges,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    for forced_range in forced_ranges {
        let (start, end) = match checked_source_range(source.text(), forced_range) {
            Ok(range) => range,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let first_max_height = finite_non_negative(max_height_for_column(forced_range, 0));
        let continuation_max_height = finite_non_negative(max_height_for_column(forced_range, 1));
        let mut max_height = first_max_height;
        let Some(text) = source.text().get(start..end) else {
            return TextShapingOutcome::failed(
                crate::core::framework::text::TextLayoutError::LayoutFailed,
            );
        };
        let chunks = match if matches!(style.wrap, TextWrap::WordSmart) {
            word_smart_line_break_chunks_with_provider(text, style, provider)
        } else {
            line_break_chunks_with_provider(text, style, provider)
        } {
            TextShapingOutcome::Ready(chunks) => chunks,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        if chunks.is_empty() {
            ranges.push(forced_range);
            continue;
        }

        let mut column_start = start;
        let mut column_end = start;
        for chunk in chunks {
            let Some(mut chunk_start) = start.checked_add(chunk.source_range.start) else {
                return TextShapingOutcome::failed(
                    crate::core::framework::text::TextLayoutError::LayoutFailed,
                );
            };
            let Some(chunk_end) = start.checked_add(chunk.source_range.end) else {
                return TextShapingOutcome::failed(
                    crate::core::framework::text::TextLayoutError::LayoutFailed,
                );
            };
            if chunk_start > chunk_end || chunk_end > end {
                return TextShapingOutcome::failed(
                    crate::core::framework::text::TextLayoutError::LayoutFailed,
                );
            }
            if column_end == column_start {
                chunk_start = match trim_rich_leading_spaces(source.text(), chunk_start, chunk_end)
                {
                    Ok(start) => start,
                    Err(error) => return TextShapingOutcome::failed(error),
                };
                column_start = chunk_start;
                column_end = chunk_start;
            }
            if chunk_start >= chunk_end {
                continue;
            }
            let break_suffix = chunk.break_suffix.map(|suffix| suffix.marker_text());
            let mut candidate_height = match advance_index.corrected_advance_with_provider(
                source.text(),
                column_start,
                chunk_end,
                break_suffix,
                provider,
            ) {
                TextShapingOutcome::Ready(height) => height,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            if column_end > column_start && candidate_height > max_height {
                if let Err(error) =
                    push_checked_range(&mut ranges, source.text(), column_start, column_end)
                {
                    return TextShapingOutcome::failed(error);
                }
                chunk_start = match trim_rich_leading_spaces(source.text(), chunk_start, chunk_end)
                {
                    Ok(start) => start,
                    Err(error) => return TextShapingOutcome::failed(error),
                };
                column_start = chunk_start;
                column_end = chunk_start;
                max_height = continuation_max_height;
                candidate_height = match advance_index.corrected_advance_with_provider(
                    source.text(),
                    column_start,
                    chunk_end,
                    break_suffix,
                    provider,
                ) {
                    TextShapingOutcome::Ready(height) => height,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
            }
            if chunk_start >= chunk_end {
                continue;
            }

            if column_end == column_start
                && candidate_height > max_height
                && chunk.allow_glyph_fallback
            {
                let fallback = match advance_index.corrected_glyph_ranges_with_provider(
                    source.text(),
                    chunk_start,
                    chunk_end,
                    max_height,
                    continuation_max_height,
                    provider,
                ) {
                    TextShapingOutcome::Ready(ranges) => ranges,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                let fallback_count = fallback.len();
                for (index, range) in fallback.into_iter().enumerate() {
                    if let Err(error) = checked_source_range(source.text(), range) {
                        return TextShapingOutcome::failed(error);
                    }
                    if index + 1 == fallback_count {
                        let (fallback_start, fallback_end) =
                            match checked_source_range(source.text(), range) {
                                Ok(range) => range,
                                Err(error) => return TextShapingOutcome::failed(error),
                            };
                        column_start = fallback_start;
                        column_end = fallback_end;
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
            if let Err(error) =
                push_checked_range(&mut ranges, source.text(), column_start, column_end)
            {
                return TextShapingOutcome::failed(error);
            }
        }
    }
    TextShapingOutcome::Ready(ranges)
}

fn trim_rich_leading_spaces(
    text: &str,
    start: usize,
    end: usize,
) -> Result<usize, crate::core::framework::text::TextLayoutError> {
    let Some(candidate) = text.get(start..end) else {
        return Err(crate::core::framework::text::TextLayoutError::LayoutFailed);
    };
    Ok(trim_leading_wrap_spaces(candidate, start).1)
}

fn push_checked_range(
    ranges: &mut Vec<(u32, u32)>,
    text: &str,
    start: usize,
    end: usize,
) -> Result<(), crate::core::framework::text::TextLayoutError> {
    ranges.push(checked_source_range_to_u32(text, start, end)?);
    Ok(())
}

fn inline_size(inline: &InlineObjectRef, _style: &TextStyle) -> (f32, f32) {
    let size = match inline {
        InlineObjectRef::Image { size, .. }
        | InlineObjectRef::Icon { size, .. }
        | InlineObjectRef::Widget { size, .. } => *size,
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

#[cfg(test)]
mod tests;
