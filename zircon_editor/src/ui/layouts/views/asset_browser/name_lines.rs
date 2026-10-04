use crate::ui::retained_host::measure_runtime_text_width;

const NAME_SINGLE_LINE_LIMIT: usize = 20;
const NAME_MIN_LINE_CHARS: usize = 6;
const NAME_TARGET_MIN_CHARS: usize = 12;
const NAME_TARGET_MAX_CHARS: usize = 18;
const MEASURE_EPSILON: f32 = 0.01;

#[derive(Clone, Copy)]
pub(super) struct RuntimeNameLineSplit {
    /// Callers pass their own text slot and font metrics so thumbnail tiles and
    /// list/table summaries can share split policy without sharing fixed widths.
    pub(super) max_width: f32,
    pub(super) primary_font_size: f32,
    pub(super) continuation_font_size: f32,
}

pub(super) fn split_display_name_lines(
    display_name: &str,
    split: RuntimeNameLineSplit,
) -> (String, String) {
    let name = display_name.trim();
    let char_count = name.chars().count();
    if char_count <= NAME_SINGLE_LINE_LIMIT
        && runtime_name_line_fits(name, split.primary_font_size, split.max_width)
    {
        return (name.to_string(), String::new());
    }

    let split_byte = name_split_byte(name, char_count, split);
    let (first, second) = name.split_at(split_byte);
    let first = name_line_text(first);
    let second = name_line_text(second);
    if first.is_empty() || second.is_empty() {
        let fallback_byte = byte_index_at_char(name, name_target(char_count));
        let (fallback_first, fallback_second) = name.split_at(fallback_byte);
        return (
            fallback_first.trim().to_string(),
            fallback_second.trim().to_string(),
        );
    }

    (first, second)
}

fn name_split_byte(name: &str, char_count: usize, split: RuntimeNameLineSplit) -> usize {
    let target = name_target(char_count);
    let mut best = None;
    let mut previous = None;
    for (split_char, (split_byte, ch)) in name.char_indices().enumerate() {
        let preferred_boundary = is_name_separator(ch)
            || previous.is_some_and(|previous: char| {
                ch.is_ascii_uppercase()
                    && (previous.is_ascii_lowercase() || previous.is_ascii_digit())
            });
        previous = Some(ch);

        if !is_valid_name_break(split_char, char_count) {
            continue;
        }
        let score = name_split_score(
            name,
            split_byte,
            split_char,
            target,
            split,
            preferred_boundary,
        );
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score < *best_score)
        {
            best = Some((score, split_byte));
        }
    }
    best.map_or_else(|| byte_index_at_char(name, target), |(_, byte)| byte)
}

fn name_target(char_count: usize) -> usize {
    (char_count / 2).clamp(NAME_TARGET_MIN_CHARS, NAME_TARGET_MAX_CHARS)
}

fn name_split_score(
    name: &str,
    split_byte: usize,
    split_char: usize,
    target: usize,
    split: RuntimeNameLineSplit,
    preferred_boundary: bool,
) -> (u32, u8, u32, usize, bool) {
    let (first, second) = name.split_at(split_byte);
    let first = trim_name_line(first);
    let second = trim_name_line(second);
    let first_width = measure_runtime_text_width(first, split.primary_font_size);
    let second_width = measure_runtime_text_width(second, split.continuation_font_size);
    let overflow =
        (first_width - split.max_width).max(0.0) + (second_width - split.max_width).max(0.0);
    let balance = (first_width - second_width).abs();

    // Avoid clipping first, then preserve authored word/camel boundaries, then
    // choose the visually most balanced two-line title.
    (
        width_score(overflow),
        u8::from(!preferred_boundary),
        width_score(balance),
        split_char.abs_diff(target),
        split_char > target,
    )
}

fn width_score(width: f32) -> u32 {
    (width.max(0.0) * 1000.0).round() as u32
}

fn is_valid_name_break(index: usize, char_count: usize) -> bool {
    index >= NAME_MIN_LINE_CHARS && char_count.saturating_sub(index) >= NAME_MIN_LINE_CHARS
}

fn runtime_name_line_fits(text: &str, font_size: f32, max_width: f32) -> bool {
    measure_runtime_text_width(text, font_size) <= max_width + MEASURE_EPSILON
}

fn is_name_separator(ch: char) -> bool {
    matches!(ch, '_' | '-' | '.' | '/' | '\\')
}

fn name_line_text(text: &str) -> String {
    trim_name_line(text).to_string()
}

fn trim_name_line(text: &str) -> &str {
    text.trim_matches(|ch: char| ch.is_whitespace() || is_name_separator(ch))
}

fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

#[cfg(test)]
#[path = "tests/name_lines.rs"]
mod tests;
