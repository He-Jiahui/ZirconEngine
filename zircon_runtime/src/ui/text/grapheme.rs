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
    if offset > 0 {
        let bytes = text.as_bytes();
        if bytes[offset - 1].is_ascii() && (offset == 1 || bytes[offset - 2].is_ascii()) {
            return Some(
                if offset >= 2 && bytes[offset - 2] == b'\r' && bytes[offset - 1] == b'\n' {
                    offset - 2
                } else {
                    offset - 1
                },
            );
        }
    }
    grapheme_indices(text)
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
}

pub(crate) fn next_grapheme_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = clamp_utf8_boundary(text, offset);
    let bytes = text.as_bytes();
    if offset < bytes.len()
        && bytes[offset].is_ascii()
        && (offset + 1 == bytes.len() || bytes[offset + 1].is_ascii())
    {
        return Some(
            if bytes[offset] == b'\r' && bytes.get(offset + 1) == Some(&b'\n') {
                offset + 2
            } else {
                offset + 1
            },
        );
    }
    grapheme_indices(text)
        .map(|(index, grapheme)| index + grapheme.len())
        .find(|end| *end > offset)
}

/// Floors an external byte offset to a user-perceived character boundary.
///
/// UI state may enter through component metadata, platform IME, accessibility, or pointer input.
/// Those consumers share this conversion so no selection, composition, or replacement range can
/// start inside a combining sequence or an emoji ZWJ cluster.
pub(crate) fn clamp_grapheme_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    if offset == 0 || offset == text.len() {
        return offset;
    }
    let bytes = text.as_bytes();
    if bytes[offset - 1].is_ascii() && bytes[offset].is_ascii() {
        return if bytes[offset - 1] == b'\r' && bytes[offset] == b'\n' {
            offset - 1
        } else {
            offset
        };
    }
    grapheme_indices(text)
        .map(|(index, _)| index)
        .take_while(|index| *index <= offset)
        .last()
        .unwrap_or(0)
}

pub(crate) fn previous_word_boundary(text: &str, offset: usize) -> Option<usize> {
    crate::text::WordBoundaryMap::new(text).previous_word_start(offset)
}

pub(crate) fn next_word_boundary(text: &str, offset: usize) -> Option<usize> {
    crate::text::WordBoundaryMap::new(text).next_word_end(offset)
}

pub(crate) fn word_range_at(text: &str, offset: usize) -> Option<(usize, usize)> {
    crate::text::WordBoundaryMap::new(text)
        .word_range_at(offset)
        .map(|range| (range.start, range.end))
}

pub(crate) fn line_start_boundary(text: &str, offset: usize) -> usize {
    crate::text::hard_line_start(text, offset)
}

pub(crate) fn line_end_boundary(text: &str, offset: usize) -> usize {
    crate::text::hard_line_end(text, offset)
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
    let next_start = crate::text::next_hard_line_start(text, current_end)?;
    let next_end = line_end_boundary(text, next_start);
    Some(line_boundary_for_grapheme_column(
        text, next_start, next_end, column,
    ))
}

pub(super) fn leading_grapheme_continuation_len(previous_text: &str, next_text: &str) -> usize {
    if previous_text.is_empty() || next_text.is_empty() {
        return 0;
    }

    // Adjacent ASCII scalars only share a grapheme when they form CRLF. Avoid
    // copying and segmenting the accumulated line for ordinary text runs.
    let previous_last = previous_text.as_bytes()[previous_text.len() - 1];
    let next_first = next_text.as_bytes()[0];
    if previous_last.is_ascii() && next_first.is_ascii() {
        return usize::from(previous_last == b'\r' && next_first == b'\n');
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

#[cfg(test)]
#[path = "grapheme/tests/cases.rs"]
mod tests;
