use crate::graphics::text::layout::{
    line_break_chunks, line_metrics, measure_line_width,
    measure_text_size as measure_backend_text_size, measured_grapheme_widths, TextLineMetrics,
};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextAlign,
    UiTextDirection, UiTextOverflow, UiTextRange, UiTextRunKind, UiTextWrap,
};

use super::grapheme::{grapheme_count, grapheme_indices, leading_grapheme_continuation_len};
use super::rich_text::{parse_source_runs, UiTextSourceRun};

mod visual_order;

#[derive(Clone, Debug)]
struct CandidateLine {
    text: String,
    source_range: UiTextRange,
    runs: Vec<UiResolvedTextRun>,
    pending_break_suffix: Option<PendingBreakSuffix>,
}

#[derive(Clone, Debug)]
struct PendingBreakSuffix {
    kind: UiTextRunKind,
    text: &'static str,
    source_range: UiTextRange,
}

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    measure_backend_text_size(text, style)
}

pub(crate) fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    let font_size = style.font_size.max(1.0);
    let metrics: TextLineMetrics = line_metrics(style);
    let line_height = metrics.line_height;
    let direction = resolve_direction(text, style.text_direction);
    let source_runs = parse_source_runs(text, style.rich_text);
    let max_width = frame.width.max(text_advance(font_size));
    let mut lines = wrap_source_runs(&source_runs, style.wrap, max_width, style);
    let clip = clip_frame.unwrap_or(frame);
    let line_capacity = (frame.height.max(line_height) / line_height)
        .floor()
        .max(1.0) as usize;
    let mut overflow_clipped = lines.len() > line_capacity;
    if matches!(style.text_overflow, UiTextOverflow::Ellipsis) && overflow_clipped {
        lines.truncate(line_capacity);
        if let Some(last) = lines.last_mut() {
            ellipsize_line(last, max_width, style);
        }
    }
    for line in &mut lines {
        visual_order::apply_visual_order(line, direction);
    }

    let mut resolved_lines = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let y = frame.y + index as f32 * line_height;
        let measured_width = measure_line_width(&line.text, style);
        let glyph_advances = measured_grapheme_widths(&line.text, style);
        let line_width = measured_width.min(frame.width.max(0.0));
        let line_frame = UiFrame::new(
            aligned_x(frame, line_width, style.text_align, direction),
            y,
            line_width,
            line_height,
        );
        if line_frame.intersection(clip).is_some() {
            resolved_lines.push(UiResolvedTextLine {
                text: line.text.clone(),
                frame: line_frame,
                source_range: line.source_range,
                visual_range: UiTextRange {
                    start: 0,
                    end: line.text.len(),
                },
                measured_width,
                glyph_advances,
                baseline: metrics.baseline,
                direction,
                runs: line.runs.clone(),
                ellipsized: line.text.ends_with('…'),
            });
        } else {
            overflow_clipped = true;
        }
    }

    let measured_width = resolved_lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    let measured_height = resolved_lines.len() as f32 * line_height;
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        overflow: style.text_overflow,
        font_size,
        line_height,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        lines: resolved_lines,
        overflow_clipped,
        editable: None,
    }
}

fn wrap_source_runs(
    runs: &[UiTextSourceRun],
    wrap: UiTextWrap,
    max_width: f32,
    style: &UiResolvedStyle,
) -> Vec<CandidateLine> {
    let mut lines = Vec::new();
    let mut current = CandidateLine {
        text: String::new(),
        source_range: UiTextRange::default(),
        runs: Vec::new(),
        pending_break_suffix: None,
    };

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
        lines.push(CandidateLine {
            text: String::new(),
            source_range: UiTextRange::default(),
            runs: Vec::new(),
            pending_break_suffix: None,
        });
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
) {
    for chunk in line_break_chunks(text, style) {
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
        if !current.text.is_empty() && !appended_text_fits(current, word_text, max_width, style) {
            trim_word_break_trailing_spaces(current);
            push_wrapped_line(lines, current);
            (word_text, word_source_range.start) =
                trim_leading_wrap_spaces(word_text, word_source_range.start);
            if word_text.is_empty() {
                continue;
            }
        }
        if chunk.allow_glyph_fallback
            && !line_text_fits(word_text, max_width, style)
            && grapheme_count(word_text) > 1
        {
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
        if !current.text.is_empty() && !appended_text_fits(current, grapheme, max_width, style) {
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

fn append_segment(
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    source_range: UiTextRange,
) {
    if text.is_empty() {
        return;
    }
    let visual_start = current.text.len();
    current.text.push_str(text);
    let visual_end = current.text.len();
    if current.runs.is_empty() {
        current.source_range.start = source_range.start;
    }
    current.source_range.end = source_range.end;
    current.runs.push(UiResolvedTextRun {
        kind,
        text: text.to_string(),
        source_range,
        visual_range: UiTextRange {
            start: visual_start,
            end: visual_end,
        },
        direction: resolve_direction(text, UiTextDirection::Auto),
    });
}

fn push_current_line(lines: &mut Vec<CandidateLine>, current: &mut CandidateLine) {
    if !current.text.is_empty() || !lines.is_empty() {
        current.pending_break_suffix = None;
        lines.push(std::mem::replace(
            current,
            CandidateLine {
                text: String::new(),
                source_range: UiTextRange::default(),
                runs: Vec::new(),
                pending_break_suffix: None,
            },
        ));
    }
}

fn push_wrapped_line(lines: &mut Vec<CandidateLine>, current: &mut CandidateLine) {
    append_pending_break_suffix(current);
    push_current_line(lines, current);
}

fn append_pending_break_suffix(current: &mut CandidateLine) {
    let Some(suffix) = current.pending_break_suffix.take() else {
        return;
    };
    append_segment(current, suffix.kind, suffix.text, suffix.source_range);
}

fn trim_leading_wrap_spaces(text: &str, source_start: usize) -> (&str, usize) {
    let trimmed = text.trim_start_matches(' ');
    (trimmed, source_start + text.len() - trimmed.len())
}

fn trim_word_break_trailing_spaces(line: &mut CandidateLine) {
    while line.text.ends_with(' ') {
        line.text.pop();
        let Some(last_run) = line.runs.last_mut() else {
            break;
        };
        if !last_run.text.ends_with(' ') {
            break;
        }
        last_run.text.pop();
        last_run.source_range.end = last_run.source_range.end.saturating_sub(1);
        last_run.visual_range.end = last_run.visual_range.end.saturating_sub(1);
        if last_run.text.is_empty() {
            line.runs.pop();
        }
    }
    line.source_range.end = line
        .runs
        .last()
        .map(|run| run.source_range.end)
        .unwrap_or(line.source_range.start);
}

fn ellipsize_line(line: &mut CandidateLine, max_width: f32, style: &UiResolvedStyle) {
    let ellipsis = "…";
    let mut text = String::new();
    let mut runs = Vec::new();

    'runs: for run in &line.runs {
        for (byte_index, grapheme) in grapheme_indices(&run.text) {
            let end = byte_index + grapheme.len();
            let continues_cluster = leading_grapheme_continuation_len(&text, grapheme) > 0;
            if continues_cluster
                || ellipsis_candidate_fits(&text, grapheme, ellipsis, max_width, style)
            {
                push_ellipsis_fragment(&mut text, &mut runs, run, byte_index, end);
                continue;
            }
            break 'runs;
        }
    }

    let visual_start = text.len();
    text.push_str(ellipsis);
    runs.push(UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: ellipsis.to_string(),
        source_range: UiTextRange {
            start: line.source_range.end,
            end: line.source_range.end,
        },
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction: resolve_direction(ellipsis, UiTextDirection::Auto),
    });
    line.text = text;
    line.runs = runs;
}

fn push_ellipsis_fragment(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
) {
    if start >= end {
        return;
    }
    let fragment = &run.text[start..end];
    let visual_start = text.len();
    text.push_str(fragment);
    runs.push(UiResolvedTextRun {
        kind: run.kind,
        text: fragment.to_string(),
        source_range: source_subrange(run.source_range, run.text.len(), start, end),
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction: resolve_direction(fragment, UiTextDirection::Auto),
    });
}

fn source_subrange(
    source_range: UiTextRange,
    visual_len: usize,
    start: usize,
    end: usize,
) -> UiTextRange {
    if source_range.start == source_range.end {
        return source_range;
    }
    if source_range.end.saturating_sub(source_range.start) != visual_len {
        return source_range;
    }
    UiTextRange {
        start: source_range.start + start,
        end: source_range.start + end,
    }
}

fn appended_text_fits(
    current: &CandidateLine,
    text: &str,
    max_width: f32,
    style: &UiResolvedStyle,
) -> bool {
    let mut candidate = String::with_capacity(current.text.len() + text.len());
    candidate.push_str(&current.text);
    candidate.push_str(text);
    line_text_fits(&candidate, max_width, style)
}

fn line_text_fits(text: &str, max_width: f32, style: &UiResolvedStyle) -> bool {
    measure_line_width(text, style) <= max_width + 0.01
}

fn ellipsis_candidate_fits(
    current: &str,
    fragment: &str,
    ellipsis: &str,
    max_width: f32,
    style: &UiResolvedStyle,
) -> bool {
    let mut candidate = String::with_capacity(current.len() + fragment.len() + ellipsis.len());
    candidate.push_str(current);
    candidate.push_str(fragment);
    candidate.push_str(ellipsis);
    line_text_fits(&candidate, max_width, style)
}

pub(super) fn text_advance(font_size: f32) -> f32 {
    (font_size.max(1.0) * 0.56).max(1.0)
}

fn aligned_x(
    frame: UiFrame,
    line_width: f32,
    align: UiTextAlign,
    direction: UiTextDirection,
) -> f32 {
    match align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - line_width) * 0.5,
        UiTextAlign::Right => frame.right() - line_width,
        UiTextAlign::Start if is_rtl_direction(direction) => frame.right() - line_width,
        UiTextAlign::Start => frame.x,
        UiTextAlign::End if is_rtl_direction(direction) => frame.x,
        UiTextAlign::End => frame.right() - line_width,
    }
}

fn is_rtl_direction(direction: UiTextDirection) -> bool {
    matches!(direction, UiTextDirection::RightToLeft)
}

fn resolve_direction(text: &str, requested: UiTextDirection) -> UiTextDirection {
    match requested {
        UiTextDirection::LeftToRight | UiTextDirection::RightToLeft => requested,
        UiTextDirection::Auto | UiTextDirection::Mixed => {
            first_strong_direction(text).unwrap_or(UiTextDirection::LeftToRight)
        }
    }
}

// UAX#9 P2/P3 paragraph direction: use the first strong character until full
// bidi level resolution replaces this low-fidelity visual-order scaffold.
fn first_strong_direction(text: &str) -> Option<UiTextDirection> {
    text.chars().find_map(|ch| {
        if is_rtl_char(ch) {
            Some(UiTextDirection::RightToLeft)
        } else if is_ltr_char(ch) {
            Some(UiTextDirection::LeftToRight)
        } else {
            None
        }
    })
}

fn is_ltr_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch.is_ascii_digit()
}

fn is_rtl_char(ch: char) -> bool {
    matches!(ch as u32, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF)
}

#[cfg(test)]
mod tests;
