use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextDirection,
    UiTextOverflow, UiTextRange, UiTextRunKind, UiTextWrap,
};

use super::rich_text::{parse_source_runs, UiTextSourceRun};

#[derive(Clone, Debug)]
struct CandidateLine {
    text: String,
    source_range: UiTextRange,
    runs: Vec<UiResolvedTextRun>,
}

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    let font_size = style.font_size.max(1.0);
    let line_height = style.line_height.max(font_size);
    let char_advance = text_advance(font_size);
    let width = text
        .lines()
        .map(|line| measure_width(line, char_advance))
        .fold(0.0_f32, f32::max);
    let line_count = text.lines().count().max(1) as f32;
    UiSize::new(width, line_height * line_count)
}

pub fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    let font_size = style.font_size.max(1.0);
    let line_height = style.line_height.max(font_size);
    let char_advance = text_advance(font_size);
    let direction = resolve_direction(text, style.text_direction);
    let source_runs = parse_source_runs(text, style.rich_text);
    let max_width = frame.width.max(char_advance);
    let mut lines = wrap_source_runs(&source_runs, style.wrap, max_width, char_advance);
    let clip = clip_frame.unwrap_or(frame);
    let line_capacity = (frame.height.max(line_height) / line_height)
        .floor()
        .max(1.0) as usize;
    let mut overflow_clipped = lines.len() > line_capacity;
    if matches!(style.text_overflow, UiTextOverflow::Ellipsis) && overflow_clipped {
        lines.truncate(line_capacity);
        if let Some(last) = lines.last_mut() {
            ellipsize_line(last, max_width, char_advance);
        }
    }

    let mut resolved_lines = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let y = frame.y + index as f32 * line_height;
        let measured_width = measure_width(&line.text, char_advance);
        let line_width = measured_width.min(frame.width.max(0.0));
        let line_frame = UiFrame::new(
            aligned_x(frame, line_width, style.text_align),
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
                baseline: font_size * 0.8,
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
    char_advance: f32,
) -> Vec<CandidateLine> {
    let max_chars = (max_width / char_advance).floor().max(1.0) as usize;
    let mut lines = Vec::new();
    let mut current = CandidateLine {
        text: String::new(),
        source_range: UiTextRange::default(),
        runs: Vec::new(),
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
                    max_chars,
                ),
                UiTextWrap::Glyph => append_glyph_wrapped_segment(
                    &mut lines,
                    &mut current,
                    run.kind,
                    &segment.text,
                    segment.range,
                    max_chars,
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
    max_chars: usize,
) {
    let mut byte_start = 0;
    for word in text.split_inclusive(' ') {
        let word_len = word.chars().count();
        if !current.text.is_empty() && current.text.chars().count() + word_len > max_chars {
            push_current_line(lines, current);
        }
        if word_len > max_chars {
            append_glyph_wrapped_segment(
                lines,
                current,
                kind,
                word,
                UiTextRange {
                    start: range.start + byte_start,
                    end: range.start + byte_start + word.len(),
                },
                max_chars,
            );
        } else {
            append_segment(
                current,
                kind,
                word,
                UiTextRange {
                    start: range.start + byte_start,
                    end: range.start + byte_start + word.len(),
                },
            );
        }
        byte_start += word.len();
    }
}

fn append_glyph_wrapped_segment(
    lines: &mut Vec<CandidateLine>,
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    range: UiTextRange,
    max_chars: usize,
) {
    for (offset, ch) in text.char_indices() {
        if current.text.chars().count() >= max_chars {
            push_current_line(lines, current);
        }
        append_segment(
            current,
            kind,
            &ch.to_string(),
            UiTextRange {
                start: range.start + offset,
                end: range.start + offset + ch.len_utf8(),
            },
        );
    }
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
        lines.push(std::mem::replace(
            current,
            CandidateLine {
                text: String::new(),
                source_range: UiTextRange::default(),
                runs: Vec::new(),
            },
        ));
    }
}

fn ellipsize_line(line: &mut CandidateLine, max_width: f32, char_advance: f32) {
    let ellipsis = "…";
    let max_chars = (max_width / char_advance).floor().max(1.0) as usize;
    let keep_chars = max_chars.saturating_sub(1);
    line.text = line.text.chars().take(keep_chars).collect::<String>();
    line.text.push_str(ellipsis);
    line.runs.clear();
    line.runs.push(UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: line.text.clone(),
        source_range: line.source_range,
        visual_range: UiTextRange {
            start: 0,
            end: line.text.len(),
        },
        direction: resolve_direction(&line.text, UiTextDirection::Auto),
    });
}

fn measure_width(text: &str, char_advance: f32) -> f32 {
    text.chars().count() as f32 * char_advance
}

fn text_advance(font_size: f32) -> f32 {
    (font_size * 0.5).max(1.0)
}

fn aligned_x(
    frame: UiFrame,
    line_width: f32,
    align: zircon_runtime_interface::ui::surface::UiTextAlign,
) -> f32 {
    match align {
        zircon_runtime_interface::ui::surface::UiTextAlign::Left => frame.x,
        zircon_runtime_interface::ui::surface::UiTextAlign::Center => {
            frame.x + (frame.width - line_width) * 0.5
        }
        zircon_runtime_interface::ui::surface::UiTextAlign::Right => frame.right() - line_width,
    }
}

fn resolve_direction(text: &str, requested: UiTextDirection) -> UiTextDirection {
    if !matches!(requested, UiTextDirection::Auto) {
        return requested;
    }
    let has_ltr = text.chars().any(is_ltr_char);
    let has_rtl = text.chars().any(is_rtl_char);
    match (has_ltr, has_rtl) {
        (true, true) => UiTextDirection::Mixed,
        (false, true) => UiTextDirection::RightToLeft,
        _ => UiTextDirection::LeftToRight,
    }
}

fn is_ltr_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch.is_ascii_digit()
}

fn is_rtl_char(ch: char) -> bool {
    matches!(ch as u32, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF)
}
