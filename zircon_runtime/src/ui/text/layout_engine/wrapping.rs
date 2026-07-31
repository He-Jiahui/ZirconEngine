use crate::text::layout::{
    corrected_glyph_ranges_with_provider, line_break_chunks_with_provider,
    line_text_fits_with_provider as shared_line_text_fits_with_provider,
    should_wrap_before_accumulated, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider, GraphemeAdvanceIndex,
};
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextRange, UiTextRunKind, UiTextWrap,
};

use super::super::adapter::text_style;
use super::super::grapheme::leading_grapheme_continuation_len;
use super::super::rich_text::UiTextSourceRun;
use super::candidate_line::{
    append_segment, push_current_line, push_wrapped_line, trim_word_break_trailing_spaces,
    CandidateLine, PendingBreakSuffix,
};
use super::direction::resolve_direction;

pub(super) fn wrap_source_runs_with_provider(
    runs: &[UiTextSourceRun],
    wrap: UiTextWrap,
    max_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> Vec<CandidateLine> {
    wrap_source_runs_with_line_widths_provider(runs, wrap, max_width, max_width, style, provider)
}

pub(super) fn wrap_source_runs_with_line_widths_provider(
    runs: &[UiTextSourceRun],
    wrap: UiTextWrap,
    first_line_width: f32,
    continuation_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> Vec<CandidateLine> {
    let mut lines = Vec::new();
    let mut current = CandidateLine::empty();
    let mut current_advance = 0.0_f32;

    for run in runs {
        for segment in split_preserving_newline(&run.text, run.source_range.start) {
            if segment.text == "\n" {
                push_current_line(&mut lines, &mut current);
                current_advance = 0.0;
                continue;
            }
            match wrap {
                UiTextWrap::None => {
                    append_segment(&mut current, run.kind, &segment.text, segment.range)
                }
                UiTextWrap::Word => append_word_wrapped_segment(
                    &mut lines,
                    &mut current,
                    run.kind,
                    &segment.text,
                    segment.range,
                    first_line_width,
                    continuation_width,
                    style,
                    provider,
                    &mut current_advance,
                    false,
                ),
                UiTextWrap::WordSmart => append_word_wrapped_segment(
                    &mut lines,
                    &mut current,
                    run.kind,
                    &segment.text,
                    segment.range,
                    first_line_width,
                    continuation_width,
                    style,
                    provider,
                    &mut current_advance,
                    true,
                ),
                UiTextWrap::Glyph => append_glyph_wrapped_segment(
                    &mut lines,
                    &mut current,
                    run.kind,
                    &segment.text,
                    segment.range,
                    first_line_width,
                    continuation_width,
                    style,
                    provider,
                    &mut current_advance,
                ),
            }
        }
    }

    push_current_line(&mut lines, &mut current);
    if lines.is_empty() {
        lines.push(CandidateLine::empty());
    }
    lines
}

#[derive(Clone)]
struct TextSegment<'a> {
    text: &'a str,
    range: UiTextRange,
}

fn split_preserving_newline(text: &str, source_start: usize) -> Vec<TextSegment<'_>> {
    let mut segments = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            if start < index {
                segments.push(TextSegment {
                    text: &text[start..index],
                    range: UiTextRange {
                        start: source_start + start,
                        end: source_start + index,
                    },
                });
            }
            segments.push(TextSegment {
                text: &text[index..index + ch.len_utf8()],
                range: UiTextRange {
                    start: source_start + index,
                    end: source_start + index + ch.len_utf8(),
                },
            });
            start = index + ch.len_utf8();
        }
    }
    if start < text.len() || segments.is_empty() {
        segments.push(TextSegment {
            text: &text[start..],
            range: UiTextRange {
                start: source_start + start,
                end: source_start + text.len(),
            },
        });
    }
    segments
}

fn append_word_wrapped_segment(
    lines: &mut Vec<CandidateLine>,
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    range: UiTextRange,
    first_line_width: f32,
    continuation_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
    current_advance: &mut f32,
    word_smart: bool,
) {
    let neutral_style = text_style(style);
    let chunks = if word_smart {
        word_smart_line_break_chunks_with_provider(text, &neutral_style, provider)
    } else {
        line_break_chunks_with_provider(text, &neutral_style, provider)
    };
    let advance_index =
        GraphemeAdvanceIndex::measured_with_provider(text, &neutral_style, provider);
    let direction = resolve_direction(text, style.text_direction).into();
    let mut segment_line_start = None;
    for chunk in chunks {
        let max_width = current_line_width(lines, first_line_width, continuation_width);
        let mut word_text = chunk.text;
        let mut word_source_range = UiTextRange {
            start: range.start + chunk.source_range.start,
            end: range.start + chunk.source_range.end,
        };
        if current.text.is_empty() {
            (word_text, word_source_range.start) =
                trim_leading_wrap_spaces(word_text, word_source_range.start);
            segment_line_start = Some(word_source_range.start.saturating_sub(range.start));
        }
        let continuation_start = word_source_range.start.saturating_sub(range.start);
        let continuation_len =
            append_leading_grapheme_continuation(current, kind, word_text, word_source_range);
        if continuation_len > 0 {
            *current_advance += advance_index.advance(
                continuation_start,
                continuation_start.saturating_add(continuation_len),
            );
            word_text = &word_text[continuation_len..];
            word_source_range.start += continuation_len;
        }
        if word_text.is_empty() {
            continue;
        }
        let mut word_advance = advance_index.advance(
            word_source_range.start.saturating_sub(range.start),
            word_source_range.end.saturating_sub(range.start),
        );
        let break_suffix = chunk.break_suffix.map(|suffix| suffix.text);
        let candidate_advance = segment_line_start.map_or(
            finite_non_negative(*current_advance) + finite_non_negative(word_advance),
            |line_start| {
                advance_index.corrected_advance_with_provider(
                    text,
                    line_start,
                    word_source_range.end.saturating_sub(range.start),
                    &neutral_style,
                    direction,
                    break_suffix,
                    provider,
                )
            },
        );
        let should_wrap = should_wrap_before_accumulated(
            current.text.is_empty(),
            0.0,
            candidate_advance,
            max_width,
        );
        let mut line_advance = segment_line_start.map(|_| candidate_advance);
        if should_wrap {
            trim_word_break_trailing_spaces(current);
            push_wrapped_line(lines, current);
            *current_advance = 0.0;
            segment_line_start = None;
            (word_text, word_source_range.start) =
                trim_leading_wrap_spaces(word_text, word_source_range.start);
            if word_text.is_empty() {
                continue;
            }
            segment_line_start = Some(word_source_range.start.saturating_sub(range.start));
            word_advance = advance_index.advance(
                word_source_range.start.saturating_sub(range.start),
                word_source_range.end.saturating_sub(range.start),
            );
            line_advance = None;
        }
        if line_advance.is_none() {
            line_advance = segment_line_start.map(|line_start| {
                advance_index.corrected_advance_with_provider(
                    text,
                    line_start,
                    word_source_range.end.saturating_sub(range.start),
                    &neutral_style,
                    direction,
                    break_suffix,
                    provider,
                )
            });
        }
        let max_width = current_line_width(lines, first_line_width, continuation_width);
        if chunk.should_fallback_to_glyph_wrap_with_advance(
            word_text,
            line_advance.unwrap_or(word_advance),
            max_width,
        ) {
            append_glyph_wrapped_segment(
                lines,
                current,
                kind,
                word_text,
                word_source_range,
                first_line_width,
                continuation_width,
                style,
                provider,
                current_advance,
            );
            segment_line_start = None;
        } else {
            append_segment(current, kind, word_text, word_source_range);
            *current_advance = line_advance.unwrap_or_else(|| {
                finite_non_negative(*current_advance) + finite_non_negative(word_advance)
            });
            current.pending_break_suffix = chunk.break_suffix.map(|suffix| PendingBreakSuffix {
                kind,
                text: suffix.text,
                source_range: UiTextRange {
                    start: range.start + suffix.source_range.start,
                    end: range.start + suffix.source_range.end,
                },
            });
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

fn append_glyph_wrapped_segment(
    lines: &mut Vec<CandidateLine>,
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    range: UiTextRange,
    first_line_width: f32,
    continuation_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
    current_advance: &mut f32,
) {
    let neutral_style = text_style(style);
    let advance_index =
        GraphemeAdvanceIndex::measured_with_provider(text, &neutral_style, provider);
    let continuation_len = append_leading_grapheme_continuation(current, kind, text, range);
    *current_advance += advance_index.advance(0, continuation_len);
    if current.text.is_empty() && continuation_len == 0 {
        let first_max_width = current_line_width(lines, first_line_width, continuation_width);
        let direction = resolve_direction(text, style.text_direction).into();
        let ranges = corrected_glyph_ranges_with_provider(
            text,
            &advance_index,
            &neutral_style,
            direction,
            first_max_width,
            continuation_width,
            provider,
        );
        for (index, (start, end)) in ranges.into_iter().enumerate() {
            if index > 0 {
                push_wrapped_line(lines, current);
            }
            for metric in advance_index.metrics_in_range(start, end) {
                let Some(grapheme) = text.get(metric.source_start..metric.source_end) else {
                    continue;
                };
                append_segment(
                    current,
                    kind,
                    grapheme,
                    UiTextRange {
                        start: range.start + metric.source_start,
                        end: range.start + metric.source_end,
                    },
                );
            }
            *current_advance = advance_index.advance(start, end);
        }
        return;
    }
    for metric in advance_index.metrics_in_range(continuation_len, text.len()) {
        let Some(grapheme) = text.get(metric.source_start..metric.source_end) else {
            continue;
        };
        let max_width = current_line_width(lines, first_line_width, continuation_width);
        if should_wrap_before_accumulated(
            current.text.is_empty(),
            *current_advance,
            metric.advance,
            max_width,
        ) {
            push_wrapped_line(lines, current);
            *current_advance = 0.0;
        }
        append_segment(
            current,
            kind,
            grapheme,
            UiTextRange {
                start: range.start + metric.source_start,
                end: range.start + metric.source_end,
            },
        );
        *current_advance += metric.advance;
    }
}

fn current_line_width(
    lines: &[CandidateLine],
    first_line_width: f32,
    continuation_width: f32,
) -> f32 {
    if lines.is_empty() {
        first_line_width
    } else {
        continuation_width
    }
}

fn append_leading_grapheme_continuation(
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    range: UiTextRange,
) -> usize {
    let continuation_len = leading_grapheme_continuation_len(&current.text, text);
    if continuation_len == 0 {
        return 0;
    }

    append_segment(
        current,
        kind,
        &text[..continuation_len],
        UiTextRange {
            start: range.start,
            end: range.start + continuation_len,
        },
    );
    continuation_len
}

pub(super) fn line_text_fits_with_provider(
    text: &str,
    max_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> bool {
    shared_line_text_fits_with_provider(text, max_width, &text_style(style), provider)
}

#[cfg(test)]
mod tests;
