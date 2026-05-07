use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextDirection,
    UiTextOverflow, UiTextRange, UiTextRunKind, UiTextWrap,
};

use super::grapheme::{
    grapheme_count, grapheme_indices, grapheme_prefix, leading_grapheme_continuation_len,
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
    for line in &mut lines {
        apply_visual_order(line, direction);
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

#[derive(Clone, Debug)]
struct VisualTextToken {
    kind: UiTextRunKind,
    text: String,
    source_range: UiTextRange,
    direction: Option<UiTextDirection>,
}

#[derive(Clone, Debug)]
struct VisualTextCluster {
    parts: Vec<VisualTextToken>,
    direction: Option<UiTextDirection>,
    neutral: bool,
}

#[derive(Clone, Debug)]
struct VisualTextFragment {
    kind: UiTextRunKind,
    text: String,
    source_range: UiTextRange,
    direction: UiTextDirection,
    neutral: bool,
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
        let mut word_text = word;
        let mut word_start = range.start + byte_start;
        if current.text.is_empty() {
            (word_text, word_start) = trim_leading_wrap_spaces(word_text, word_start);
        }
        let continuation_len = append_leading_grapheme_continuation(
            current,
            kind,
            word_text,
            UiTextRange {
                start: word_start,
                end: word_start + word_text.len(),
            },
        );
        if continuation_len > 0 {
            word_text = &word_text[continuation_len..];
            word_start += continuation_len;
        }
        let word_len = grapheme_count(word_text);
        if word_text.is_empty() {
            byte_start += word.len();
            continue;
        }
        if !current.text.is_empty() && grapheme_count(&current.text) + word_len > max_chars {
            trim_word_break_trailing_spaces(current);
            push_current_line(lines, current);
            (word_text, word_start) = trim_leading_wrap_spaces(word_text, word_start);
            if word_text.is_empty() {
                byte_start += word.len();
                continue;
            }
        }
        if word_len > max_chars {
            append_glyph_wrapped_segment(
                lines,
                current,
                kind,
                word_text,
                UiTextRange {
                    start: word_start,
                    end: word_start + word_text.len(),
                },
                max_chars,
            );
        } else {
            append_segment(
                current,
                kind,
                word_text,
                UiTextRange {
                    start: word_start,
                    end: word_start + word_text.len(),
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
    let continuation_len = append_leading_grapheme_continuation(current, kind, text, range);
    for (offset, grapheme) in grapheme_indices(&text[continuation_len..]) {
        let offset = continuation_len + offset;
        if grapheme_count(&current.text) >= max_chars {
            push_current_line(lines, current);
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

fn ellipsize_line(line: &mut CandidateLine, max_width: f32, char_advance: f32) {
    let ellipsis = "…";
    let max_chars = (max_width / char_advance).floor().max(1.0) as usize;
    let keep_chars = max_chars.saturating_sub(1);
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut remaining = keep_chars;
    for run in &line.runs {
        let mut consumed = 0;
        if remaining > 0 {
            let fragment = grapheme_prefix(&run.text, remaining);
            if !fragment.is_empty() {
                consumed = fragment.len();
                push_ellipsis_fragment(&mut text, &mut runs, run, 0, consumed);
                remaining = remaining.saturating_sub(grapheme_count(fragment));
            }
        }

        if remaining == 0 {
            if consumed == 0 {
                let continuation_len = leading_grapheme_continuation_len(&text, &run.text);
                if continuation_len > 0 {
                    consumed = continuation_len;
                    push_ellipsis_fragment(&mut text, &mut runs, run, 0, consumed);
                }
            }

            if consumed < run.text.len() {
                break;
            }
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
        source_range: source_subrange(run.source_range, start, end),
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction: resolve_direction(fragment, UiTextDirection::Auto),
    });
}

// This is a low-fidelity BiDi scaffold: it preserves source/visual byte ranges while
// deferring full glyph shaping, mirroring, and cluster handling to the text backends.
fn apply_visual_order(line: &mut CandidateLine, base_direction: UiTextDirection) {
    if line.runs.is_empty() {
        return;
    }
    let visual_fragments = visual_text_fragments(&line.runs, base_direction);
    if visual_fragments.is_empty() {
        return;
    }

    let mut visual_text = String::new();
    let mut visual_runs = Vec::with_capacity(visual_fragments.len());
    for fragment in visual_fragments {
        let visual_start = visual_text.len();
        visual_text.push_str(&fragment.text);
        visual_runs.push(UiResolvedTextRun {
            kind: fragment.kind,
            text: fragment.text,
            source_range: fragment.source_range,
            visual_range: UiTextRange {
                start: visual_start,
                end: visual_text.len(),
            },
            direction: fragment.direction,
        });
    }

    line.text = visual_text;
    line.runs = visual_runs;
}

fn visual_text_fragments(
    runs: &[UiResolvedTextRun],
    base_direction: UiTextDirection,
) -> Vec<VisualTextFragment> {
    let clusters = visual_text_clusters(runs);
    let has_rtl = clusters.iter().any(|cluster| {
        cluster
            .direction
            .is_some_and(|direction| matches!(direction, UiTextDirection::RightToLeft))
    });
    if !has_rtl {
        return runs
            .iter()
            .map(|run| VisualTextFragment {
                kind: run.kind,
                text: run.text.clone(),
                source_range: run.source_range,
                direction: run.direction,
                neutral: false,
            })
            .collect();
    }
    let clusters = assign_neutral_cluster_directions(clusters, base_direction);

    let mut spans = Vec::<Vec<VisualTextCluster>>::new();
    let mut current = Vec::<VisualTextCluster>::new();
    let mut current_direction = None;
    for cluster in clusters {
        let direction = cluster.direction.unwrap_or_else(|| {
            default_visual_direction(base_direction).unwrap_or(UiTextDirection::LeftToRight)
        });
        if current_direction.is_some_and(|current| current != direction) {
            spans.push(current);
            current = Vec::new();
        }
        current_direction = Some(direction);
        current.push(VisualTextCluster {
            direction: Some(direction),
            ..cluster
        });
    }
    if !current.is_empty() {
        spans.push(current);
    }
    if matches!(base_direction, UiTextDirection::RightToLeft) {
        spans.reverse();
    }

    let mut fragments = Vec::new();
    for mut span in spans {
        let span_direction = span
            .first()
            .and_then(|cluster| cluster.direction)
            .unwrap_or(UiTextDirection::LeftToRight);
        if matches!(span_direction, UiTextDirection::RightToLeft) {
            span.reverse();
            for cluster in span {
                push_visual_cluster(&mut fragments, cluster, UiTextDirection::RightToLeft);
            }
        } else {
            for cluster in span {
                push_visual_cluster(&mut fragments, cluster, UiTextDirection::LeftToRight);
            }
        }
    }
    fragments
}

fn assign_neutral_cluster_directions(
    mut clusters: Vec<VisualTextCluster>,
    base_direction: UiTextDirection,
) -> Vec<VisualTextCluster> {
    let fallback = default_visual_direction(base_direction).unwrap_or(UiTextDirection::LeftToRight);
    for index in 0..clusters.len() {
        if clusters[index].direction.is_some() {
            continue;
        }
        let previous = clusters[..index]
            .iter()
            .rev()
            .find_map(|cluster| cluster.direction);
        let next = clusters[index + 1..]
            .iter()
            .find_map(|cluster| cluster.direction);
        clusters[index].direction = Some(neutral_token_direction(previous, next, fallback));
    }
    clusters
}

fn neutral_token_direction(
    previous: Option<UiTextDirection>,
    next: Option<UiTextDirection>,
    fallback: UiTextDirection,
) -> UiTextDirection {
    match (previous, next) {
        (Some(previous), Some(next)) if previous == next => previous,
        // Keep LTR/RTL boundary separators on the LTR side, but let punctuation inside an
        // RTL phrase travel with the surrounding RTL span until a real shaper replaces this.
        (Some(UiTextDirection::LeftToRight), Some(UiTextDirection::RightToLeft))
        | (Some(UiTextDirection::RightToLeft), Some(UiTextDirection::LeftToRight)) => {
            UiTextDirection::LeftToRight
        }
        (Some(previous), Some(_)) => previous,
        (Some(previous), None) => previous,
        (None, Some(next)) => next,
        (None, None) => fallback,
    }
}

fn visual_text_clusters(runs: &[UiResolvedTextRun]) -> Vec<VisualTextCluster> {
    let mut clusters = Vec::new();
    let mut emitted_text = String::new();
    for run in runs {
        let mut consumed = 0;
        if !clusters.is_empty() {
            let continuation_len = leading_grapheme_continuation_len(&emitted_text, &run.text);
            if continuation_len > 0 {
                let token = visual_token(
                    run,
                    0,
                    continuation_len,
                    grapheme_direction(&run.text[..continuation_len]),
                );
                push_visual_cluster_part(clusters.last_mut().unwrap(), token);
                emitted_text.push_str(&run.text[..continuation_len]);
                consumed = continuation_len;
            }
        }

        for (offset, grapheme) in grapheme_indices(&run.text[consumed..]) {
            let offset = consumed + offset;
            let direction = grapheme_direction(grapheme);
            clusters.push(VisualTextCluster {
                parts: vec![visual_token(
                    run,
                    offset,
                    offset + grapheme.len(),
                    direction,
                )],
                direction,
                neutral: direction.is_none(),
            });
            emitted_text.push_str(grapheme);
        }
    }
    clusters.retain(|cluster| !cluster.parts.is_empty());
    clusters
}

fn visual_token(
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
    direction: Option<UiTextDirection>,
) -> VisualTextToken {
    VisualTextToken {
        kind: run.kind,
        text: run.text[start..end].to_string(),
        source_range: source_subrange(run.source_range, start, end),
        direction,
    }
}

fn push_visual_cluster_part(cluster: &mut VisualTextCluster, token: VisualTextToken) {
    if cluster.direction.is_none() {
        cluster.direction = token.direction;
    }
    if token.direction.is_some() {
        cluster.neutral = false;
    }
    cluster.parts.push(token);
}

fn push_visual_cluster(
    fragments: &mut Vec<VisualTextFragment>,
    cluster: VisualTextCluster,
    direction: UiTextDirection,
) {
    for token in cluster.parts {
        push_visual_fragment(
            fragments,
            VisualTextFragment {
                kind: token.kind,
                text: token.text,
                source_range: token.source_range,
                direction,
                neutral: cluster.neutral,
            },
        );
    }
}

fn source_subrange(source_range: UiTextRange, start: usize, end: usize) -> UiTextRange {
    if source_range.start == source_range.end {
        return source_range;
    }
    UiTextRange {
        start: source_range.start + start,
        end: source_range.start + end,
    }
}

fn push_visual_fragment(fragments: &mut Vec<VisualTextFragment>, fragment: VisualTextFragment) {
    if let Some(last) = fragments.last_mut() {
        if last.kind == fragment.kind
            && last.direction == fragment.direction
            && last.source_range.end == fragment.source_range.start
        {
            last.text.push_str(&fragment.text);
            last.source_range.end = fragment.source_range.end;
            last.neutral &= fragment.neutral;
            return;
        }
        if last.kind == fragment.kind
            && last.direction == fragment.direction
            && !last.neutral
            && !fragment.neutral
            && fragment.source_range.end == last.source_range.start
        {
            last.text.push_str(&fragment.text);
            last.source_range.start = fragment.source_range.start;
            return;
        }
    }
    fragments.push(fragment);
}

fn default_visual_direction(direction: UiTextDirection) -> Option<UiTextDirection> {
    match direction {
        UiTextDirection::LeftToRight => Some(UiTextDirection::LeftToRight),
        UiTextDirection::RightToLeft => Some(UiTextDirection::RightToLeft),
        _ => None,
    }
}

fn strong_char_direction(ch: char) -> Option<UiTextDirection> {
    if is_rtl_char(ch) {
        Some(UiTextDirection::RightToLeft)
    } else if is_ltr_char(ch) {
        Some(UiTextDirection::LeftToRight)
    } else {
        None
    }
}

fn grapheme_direction(grapheme: &str) -> Option<UiTextDirection> {
    grapheme.chars().find_map(strong_char_direction)
}

fn measure_width(text: &str, char_advance: f32) -> f32 {
    grapheme_count(text) as f32 * char_advance
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

#[cfg(test)]
mod tests;
