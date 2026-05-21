use unicode_segmentation::UnicodeSegmentation;

// Centralize Unicode text boundaries so layout and editing scaffolds do not
// split combining marks, emoji clusters, or word navigation before shaping.
pub(super) fn grapheme_count(text: &str) -> usize {
    text.graphemes(true).count()
}

pub(super) fn grapheme_indices(text: &str) -> impl Iterator<Item = (usize, &str)> + '_ {
    text.grapheme_indices(true)
}

pub(crate) fn previous_grapheme_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    grapheme_indices(text)
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
}

pub(crate) fn next_grapheme_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    grapheme_indices(text)
        .map(|(index, grapheme)| index + grapheme.len())
        .find(|end| *end > offset)
}

pub(crate) fn previous_word_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    let mut previous_word_start = None;
    for (start, segment) in text.split_word_bound_indices() {
        let end = start + segment.len();
        if end > offset {
            if start < offset && is_word_segment(segment) {
                return Some(start);
            }
            break;
        }
        if is_word_segment(segment) {
            previous_word_start = Some(start);
        }
    }
    previous_word_start
}

pub(crate) fn next_word_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    for (start, segment) in text.split_word_bound_indices() {
        let end = start + segment.len();
        if end <= offset {
            continue;
        }
        if is_word_segment(segment) {
            return Some(end);
        }
    }
    None
}

pub(crate) fn line_start_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    text[..offset]
        .rfind('\n')
        .map(|index| index + '\n'.len_utf8())
        .unwrap_or(0)
}

pub(crate) fn line_end_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    let end = text[offset..]
        .find('\n')
        .map(|relative| offset + relative)
        .unwrap_or(text.len());
    if end > 0 && text.as_bytes().get(end - 1) == Some(&b'\r') {
        end - 1
    } else {
        end
    }
}

pub(crate) fn previous_line_same_column_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    let current_start = line_start_boundary(text, offset);
    if current_start == 0 {
        return None;
    }

    let column = grapheme_column_in_line(text, current_start, offset);
    let previous_start = line_start_boundary(text, current_start.saturating_sub(1));
    let previous_end = line_end_boundary(text, previous_start);
    Some(line_boundary_for_grapheme_column(
        text,
        previous_start,
        previous_end,
        column,
    ))
}

pub(crate) fn next_line_same_column_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    let current_start = line_start_boundary(text, offset);
    let current_end = line_end_boundary(text, offset);
    let column = grapheme_column_in_line(text, current_start, offset.min(current_end));
    let next_start = next_line_start_after_end(text, current_end)?;
    let next_end = line_end_boundary(text, next_start);
    Some(line_boundary_for_grapheme_column(
        text, next_start, next_end, column,
    ))
}

pub(super) fn leading_grapheme_continuation_len(previous_text: &str, next_text: &str) -> usize {
    if previous_text.is_empty() || next_text.is_empty() {
        return 0;
    }

    let split = previous_text.len();
    let mut combined = String::with_capacity(previous_text.len() + next_text.len());
    combined.push_str(previous_text);
    combined.push_str(next_text);

    for (start, grapheme) in combined.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < split && split < end {
            return end - split;
        }
        if start >= split {
            break;
        }
    }

    0
}

pub(super) fn grapheme_prefix(text: &str, max_graphemes: usize) -> &str {
    if max_graphemes == 0 {
        return "";
    }
    let end = grapheme_indices(text)
        .map(|(index, grapheme)| index + grapheme.len())
        .nth(max_graphemes - 1)
        .unwrap_or(text.len());
    &text[..end]
}

fn clamp_utf8_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn grapheme_column_in_line(text: &str, line_start: usize, offset: usize) -> usize {
    let line_end = line_end_boundary(text, line_start);
    let offset = clamp_utf8_boundary(text, offset).min(line_end);
    text[line_start..offset].graphemes(true).count()
}

fn line_boundary_for_grapheme_column(
    text: &str,
    line_start: usize,
    line_end: usize,
    column: usize,
) -> usize {
    if column == 0 {
        return line_start;
    }
    grapheme_indices(&text[line_start..line_end])
        .map(|(index, grapheme)| line_start + index + grapheme.len())
        .nth(column - 1)
        .unwrap_or(line_end)
}

fn next_line_start_after_end(text: &str, line_end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(line_end) == Some(&b'\r') && bytes.get(line_end + 1) == Some(&b'\n') {
        return Some(line_end + 2);
    }
    if bytes.get(line_end) == Some(&b'\n') {
        return Some(line_end + 1);
    }
    None
}

fn is_word_segment(segment: &str) -> bool {
    segment.chars().any(|character| character.is_alphanumeric())
}
