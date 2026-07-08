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
    let mut candidates = Vec::new();
    collect_separator_breaks(name, char_count, &mut candidates);
    collect_camel_case_breaks(name, char_count, &mut candidates);
    collect_runtime_width_breaks(char_count, &mut candidates);
    candidates.push(target);
    candidates.sort_unstable();
    candidates.dedup();
    let split_char = candidates
        .into_iter()
        .filter(|candidate| is_valid_name_break(*candidate, char_count))
        .min_by_key(|candidate| name_split_score(name, *candidate, target, split))
        .unwrap_or(target);
    byte_index_at_char(name, split_char)
}

fn name_target(char_count: usize) -> usize {
    (char_count / 2).clamp(NAME_TARGET_MIN_CHARS, NAME_TARGET_MAX_CHARS)
}

fn collect_separator_breaks(name: &str, char_count: usize, candidates: &mut Vec<usize>) {
    for (index, ch) in name.chars().enumerate() {
        if is_name_separator(ch) && is_valid_name_break(index, char_count) {
            candidates.push(index);
        }
    }
}

fn collect_camel_case_breaks(name: &str, char_count: usize, candidates: &mut Vec<usize>) {
    let mut previous: Option<char> = None;
    for (index, ch) in name.chars().enumerate() {
        if let Some(previous) = previous {
            let is_boundary = ch.is_ascii_uppercase()
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit());
            if is_boundary && is_valid_name_break(index, char_count) {
                candidates.push(index);
            }
        }
        previous = Some(ch);
    }
}

fn collect_runtime_width_breaks(char_count: usize, candidates: &mut Vec<usize>) {
    if char_count < NAME_MIN_LINE_CHARS * 2 {
        return;
    }
    let last_valid_split = char_count - NAME_MIN_LINE_CHARS;
    candidates.extend(NAME_MIN_LINE_CHARS..=last_valid_split);
}

fn name_split_score(
    name: &str,
    split_char: usize,
    target: usize,
    split: RuntimeNameLineSplit,
) -> (u32, u8, u32, usize, bool) {
    let split_byte = byte_index_at_char(name, split_char);
    let (first, second) = name.split_at(split_byte);
    let first = name_line_text(first);
    let second = name_line_text(second);
    let first_width = measure_runtime_text_width(&first, split.primary_font_size);
    let second_width = measure_runtime_text_width(&second, split.continuation_font_size);
    let overflow =
        (first_width - split.max_width).max(0.0) + (second_width - split.max_width).max(0.0);
    let balance = (first_width - second_width).abs();

    // Avoid clipping first, then preserve authored word/camel boundaries, then
    // choose the visually most balanced two-line title.
    (
        width_score(overflow),
        break_rank(name, split_char),
        width_score(balance),
        split_char.abs_diff(target),
        split_char > target,
    )
}

fn width_score(width: f32) -> u32 {
    (width.max(0.0) * 1000.0).round() as u32
}

fn break_rank(name: &str, split_char: usize) -> u8 {
    let mut previous: Option<char> = None;
    for (index, ch) in name.chars().enumerate() {
        if index == split_char {
            if is_name_separator(ch) {
                return 0;
            }
            if let Some(previous) = previous {
                if ch.is_ascii_uppercase()
                    && (previous.is_ascii_lowercase() || previous.is_ascii_digit())
                {
                    return 0;
                }
            }
            return 1;
        }
        previous = Some(ch);
    }
    1
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
    text.trim_matches(|ch: char| ch.is_whitespace() || is_name_separator(ch))
        .to_string()
}

fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_name_lines_use_width_before_character_limit() {
        let wide_name = "WWWWWWWWWWWWWWWWWW";
        let split = RuntimeNameLineSplit {
            max_width: 96.0,
            primary_font_size: 10.0,
            continuation_font_size: 9.0,
        };
        assert!(wide_name.chars().count() <= NAME_SINGLE_LINE_LIMIT);
        assert!(
            measure_runtime_text_width(wide_name, split.primary_font_size) > split.max_width,
            "test fixture must exceed the thumbnail name slot"
        );

        let (name, continuation) = split_display_name_lines(wide_name, split);

        assert!(!continuation.is_empty());
        assert!(
            measure_runtime_text_width(name.as_str(), split.primary_font_size)
                <= split.max_width + MEASURE_EPSILON,
            "primary thumbnail title line should fit measured width: {name}"
        );
        assert!(
            measure_runtime_text_width(continuation.as_str(), split.continuation_font_size)
                <= split.max_width + MEASURE_EPSILON,
            "continuation thumbnail title line should fit measured width: {continuation}"
        );
    }

    #[test]
    fn runtime_name_lines_keep_camel_case_boundaries_when_width_allows() {
        let split = RuntimeNameLineSplit {
            max_width: 96.0,
            primary_font_size: 10.0,
            continuation_font_size: 9.0,
        };

        assert_eq!(
            split_display_name_lines("NavigationSettingsRuntimeProfile", split),
            (
                "NavigationSettings".to_string(),
                "RuntimeProfile".to_string()
            )
        );
    }
}
