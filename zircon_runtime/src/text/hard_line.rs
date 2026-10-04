use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HardLine {
    pub(crate) content: Range<usize>,
    pub(crate) separator: Range<usize>,
}

impl HardLine {
    pub(crate) fn source_range(&self) -> Range<usize> {
        self.content.start..self.separator.end.max(self.content.end)
    }
}

pub(crate) fn hard_lines(text: &str) -> Vec<HardLine> {
    let mut lines = Vec::new();
    visit_hard_lines(text, |line| {
        lines.push(line);
    });
    lines
}

/// Visits canonical hard lines without retaining a document-sized line vector.
pub(crate) fn visit_hard_lines(text: &str, mut visit: impl FnMut(HardLine)) {
    for_each_hard_line(text, |line| {
        visit(line);
        true
    });
}

/// Counts canonical hard lines without retaining a per-line allocation.
pub(crate) fn hard_line_count(text: &str) -> usize {
    let mut count: usize = 0;
    for_each_hard_line(text, |_| {
        count = count.saturating_add(1);
        true
    });
    count
}

/// Returns whether source segmentation can produce more than one hard line.
///
/// This is the allocation-free rejection path for viewport virtualization. Only a source
/// separator changes the document's hard-line identity; backend execution policy must not become
/// a layout line.
pub(crate) fn has_multiple_hard_lines(text: &str) -> bool {
    text.chars().any(is_hard_line_separator)
}

/// Materializes only the requested canonical hard-line range.
pub(crate) fn hard_line_window(text: &str, range: Range<usize>) -> Vec<HardLine> {
    if range.start >= range.end {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut line_index = 0;
    // The viewport counts with the same scanner first; stop this pass as soon as its window ends.
    for_each_hard_line(text, |line| {
        if range.contains(&line_index) {
            lines.push(line);
        }
        line_index = line_index.saturating_add(1);
        line_index < range.end
    });
    lines
}

/// Counts all canonical hard lines while retaining only the requested window.
///
/// This is the bounded viewport path: document height still needs the full count, but the
/// layout only needs candidate lines around the visible region.
pub(crate) fn hard_line_count_and_window(
    text: &str,
    range: Range<usize>,
) -> (usize, Vec<HardLine>) {
    let mut count = 0usize;
    let mut lines = Vec::new();
    for_each_hard_line(text, |line| {
        if range.contains(&count) {
            lines.push(line);
        }
        count = count.saturating_add(1);
        true
    });
    (count, lines)
}

/// Returns the source start of the canonical hard line containing `offset`.
pub(crate) fn hard_line_start(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    text[..offset]
        .char_indices()
        .rev()
        .find(|(index, character)| {
            is_hard_line_separator(*character)
                && !is_crlf_prefix_boundary(text, *index, *character, offset)
        })
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0)
}

/// Returns the source end, excluding the separator, of the canonical hard line at `offset`.
pub(crate) fn hard_line_end(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    if text.as_bytes().get(offset) == Some(&b'\n')
        && offset > 0
        && text.as_bytes().get(offset - 1) == Some(&b'\r')
    {
        return offset - 1;
    }
    text[offset..]
        .char_indices()
        .find(|(_, character)| is_hard_line_separator(*character))
        .map(|(index, _)| offset + index)
        .unwrap_or(text.len())
}

/// Returns the source start of the next canonical hard line after a line end.
pub(crate) fn next_hard_line_start(text: &str, line_end: usize) -> Option<usize> {
    let line_end = clamp_utf8_boundary(text, line_end);
    let separator = text.get(line_end..)?.chars().next()?;
    if separator == '\r' {
        return Some(
            line_end
                + if text.as_bytes().get(line_end + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                },
        );
    }
    is_hard_line_separator(separator).then_some(line_end + separator.len_utf8())
}

fn for_each_hard_line(text: &str, mut visit: impl FnMut(HardLine) -> bool) {
    let mut line_start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        let mut next_start = index + ch.len_utf8();
        let is_break = match ch {
            '\r' => {
                if let Some((next_index, '\n')) = chars.peek().copied() {
                    chars.next();
                    next_start = next_index + '\n'.len_utf8();
                }
                true
            }
            _ => is_hard_line_separator(ch),
        };
        if is_break {
            if !visit(HardLine {
                content: line_start..index,
                separator: index..next_start,
            }) {
                return;
            }
            line_start = next_start;
        }
    }
    let _ = visit(HardLine {
        content: line_start..text.len(),
        separator: text.len()..text.len(),
    });
}

pub(crate) fn is_hard_line_separator(character: char) -> bool {
    matches!(
        character,
        '\r' | '\n' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

/// A prefix ending at the LF byte of CRLF contains only an incomplete separator.
/// Treating its CR as a completed break would place reverse line navigation inside CRLF.
fn is_crlf_prefix_boundary(text: &str, index: usize, character: char, offset: usize) -> bool {
    character == '\r'
        && text.as_bytes().get(index + 1) == Some(&b'\n')
        && index.saturating_add(1) >= offset
}

fn clamp_utf8_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
#[path = "tests/hard_line.rs"]
mod tests;
