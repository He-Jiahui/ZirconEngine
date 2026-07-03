use crate::graphics::text::layout::{
    line_break_chunks, line_text_fits as shared_line_text_fits, should_wrap_before_chunk,
    trim_leading_wrap_spaces, word_smart_line_break_chunks,
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextRange, UiTextRunKind, UiTextWrap,
};

use super::super::grapheme::{grapheme_indices, leading_grapheme_continuation_len};
use super::super::rich_text::UiTextSourceRun;
use super::candidate_line::{
    append_segment, push_current_line, push_wrapped_line, trim_word_break_trailing_spaces,
    CandidateLine, PendingBreakSuffix,
};

pub(super) fn wrap_source_runs(
    runs: &[UiTextSourceRun],
    wrap: UiTextWrap,
    max_width: f32,
    style: &UiResolvedStyle,
) -> Vec<CandidateLine> {
    let mut lines = Vec::new();
    let mut current = CandidateLine::empty();

    for run in runs {
        for segment in split_preserving_newline(&run.text, run.source_range.start) {
            if segment.text == "\n" {
                push_current_line(&mut lines, &mut current);
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
                    max_width,
                    style,
                    false,
                ),
                UiTextWrap::WordSmart => append_word_wrapped_segment(
                    &mut lines,
                    &mut current,
                    run.kind,
                    &segment.text,
                    segment.range,
                    max_width,
                    style,
                    true,
                ),
                UiTextWrap::Glyph => append_glyph_wrapped_segment(
                    &mut lines,
                    &mut current,
                    run.kind,
                    &segment.text,
                    segment.range,
                    max_width,
                    style,
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
struct TextSegment {
    text: String,
    range: UiTextRange,
}

fn split_preserving_newline(text: &str, source_start: usize) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            if start < index {
                segments.push(TextSegment {
                    text: text[start..index].to_string(),
                    range: UiTextRange {
                        start: source_start + start,
                        end: source_start + index,
                    },
                });
            }
            segments.push(TextSegment {
                text: "\n".to_string(),
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
            text: text[start..].to_string(),
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
    max_width: f32,
    style: &UiResolvedStyle,
    word_smart: bool,
) {
    let chunks = if word_smart {
        word_smart_line_break_chunks(text, style)
    } else {
        line_break_chunks(text, style)
    };
    for chunk in chunks {
        let mut word_text = chunk.text;
        let mut word_source_range = UiTextRange {
            start: range.start + chunk.source_range.start,
            end: range.start + chunk.source_range.end,
        };
        if current.text.is_empty() {
            (word_text, word_source_range.start) =
                trim_leading_wrap_spaces(word_text, word_source_range.start);
        }
        let continuation_len =
            append_leading_grapheme_continuation(current, kind, word_text, word_source_range);
        if continuation_len > 0 {
            word_text = &word_text[continuation_len..];
            word_source_range.start += continuation_len;
        }
        if word_text.is_empty() {
            continue;
        }
        if should_wrap_before_chunk(&current.text, word_text, max_width, style) {
            trim_word_break_trailing_spaces(current);
            push_wrapped_line(lines, current);
            (word_text, word_source_range.start) =
                trim_leading_wrap_spaces(word_text, word_source_range.start);
            if word_text.is_empty() {
                continue;
            }
        }
        if chunk.should_fallback_to_glyph_wrap(word_text, max_width, style) {
            append_glyph_wrapped_segment(
                lines,
                current,
                kind,
                word_text,
                word_source_range,
                max_width,
                style,
            );
        } else {
            append_segment(current, kind, word_text, word_source_range);
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

fn append_glyph_wrapped_segment(
    lines: &mut Vec<CandidateLine>,
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    range: UiTextRange,
    max_width: f32,
    style: &UiResolvedStyle,
) {
    let continuation_len = append_leading_grapheme_continuation(current, kind, text, range);
    for (offset, grapheme) in grapheme_indices(&text[continuation_len..]) {
        let offset = continuation_len + offset;
        if should_wrap_before_chunk(&current.text, grapheme, max_width, style) {
            push_wrapped_line(lines, current);
        }
        append_segment(
            current,
            kind,
            grapheme,
            UiTextRange {
                start: range.start + offset,
                end: range.start + offset + grapheme.len(),
            },
        );
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

pub(super) fn line_text_fits(text: &str, max_width: f32, style: &UiResolvedStyle) -> bool {
    shared_line_text_fits(text, max_width, style)
}
