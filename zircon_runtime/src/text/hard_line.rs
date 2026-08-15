use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

/// Safety ceiling for one backend shaping run. Longer unbroken content is represented as
/// contiguous internal layout lines without changing the source text.
pub(crate) const TEXT_SHAPING_RUN_MAX_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HardLine {
    pub(crate) content: Range<usize>,
    pub(crate) separator: Range<usize>,
    run_cap_break: bool,
}

impl HardLine {
    pub(crate) fn source_range(&self) -> Range<usize> {
        self.content.start..self.separator.end.max(self.content.end)
    }

    /// A synthetic layout boundary prevents a backend from shaping an unbounded source run.
    /// It has no separator or source glyph because no source character was inserted.
    pub(crate) const fn is_run_cap_break(&self) -> bool {
        self.run_cap_break
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

/// Returns whether canonical segmentation can produce more than one hard line.
///
/// This is the allocation-free rejection path for viewport virtualization. A source separator or
/// an over-cap run is the only way the plain, unwrapped path can select a strict line subset.
pub(crate) fn has_multiple_hard_lines(text: &str) -> bool {
    text.len() > TEXT_SHAPING_RUN_MAX_BYTES || text.chars().any(is_hard_line_separator)
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
            if !visit_capped_hard_line(text, line_start..index, index..next_start, &mut visit) {
                return;
            }
            line_start = next_start;
        }
    }
    let _ = visit_capped_hard_line(
        text,
        line_start..text.len(),
        text.len()..text.len(),
        &mut visit,
    );
}

fn is_hard_line_separator(character: char) -> bool {
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

fn visit_capped_hard_line(
    text: &str,
    content: Range<usize>,
    separator: Range<usize>,
    visit: &mut impl FnMut(HardLine) -> bool,
) -> bool {
    if content.is_empty() {
        return visit(HardLine {
            content,
            separator,
            run_cap_break: false,
        });
    }

    let mut chunk_start = content.start;
    while chunk_start < content.end {
        let chunk_end = capped_content_end(text, chunk_start, content.end);
        let run_cap_break = chunk_end < content.end;
        if !visit(HardLine {
            content: chunk_start..chunk_end,
            separator: if run_cap_break {
                chunk_end..chunk_end
            } else {
                separator.clone()
            },
            run_cap_break,
        }) {
            return false;
        }
        chunk_start = chunk_end;
    }
    true
}

fn capped_content_end(text: &str, start: usize, end: usize) -> usize {
    let cap_end = start.saturating_add(TEXT_SHAPING_RUN_MAX_BYTES).min(end);
    if cap_end == end {
        return end;
    }

    let mut last_grapheme_end = start;
    for (offset, grapheme) in text[start..end].grapheme_indices(true) {
        let grapheme_end = start + offset + grapheme.len();
        if grapheme_end > cap_end {
            break;
        }
        last_grapheme_end = grapheme_end;
    }
    if last_grapheme_end > start {
        return last_grapheme_end;
    }

    // A single pathological grapheme can exceed the cap. Preserve UTF-8 validity while making
    // forward progress so it cannot retain an unbounded backend shaping path.
    let mut utf8_boundary = cap_end;
    while utf8_boundary > start && !text.is_char_boundary(utf8_boundary) {
        utf8_boundary -= 1;
    }
    debug_assert!(utf8_boundary > start);
    utf8_boundary
}

#[cfg(test)]
mod tests {
    use super::{
        hard_line_count, hard_line_count_and_window, hard_line_end, hard_line_start,
        hard_line_window, hard_lines, has_multiple_hard_lines, next_hard_line_start,
        visit_hard_lines, TEXT_SHAPING_RUN_MAX_BYTES,
    };

    #[test]
    fn hard_lines_preserve_crlf_and_unicode_separator_ranges() {
        let text = "a\r\nb\u{2028}c\n";

        let lines = hard_lines(text);

        assert_eq!(
            lines
                .iter()
                .map(|line| (line.content.clone(), line.separator.clone()))
                .collect::<Vec<_>>(),
            vec![(0..1, 1..3), (3..4, 4..7), (7..8, 8..9), (9..9, 9..9)]
        );
        assert!(lines
            .windows(2)
            .all(|lines| lines[0].source_range().end == lines[1].source_range().start));
    }

    #[test]
    fn hard_lines_cap_an_oversized_unbroken_run_at_a_utf8_boundary() {
        let mut text = "a".repeat(TEXT_SHAPING_RUN_MAX_BYTES - 1);
        text.push('中');
        text.push('b');

        let lines = hard_lines(&text);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].content, 0..TEXT_SHAPING_RUN_MAX_BYTES - 1);
        assert_eq!(
            lines[0].separator,
            TEXT_SHAPING_RUN_MAX_BYTES - 1..TEXT_SHAPING_RUN_MAX_BYTES - 1
        );
        assert!(lines[0].is_run_cap_break());
        assert_eq!(lines[1].content, TEXT_SHAPING_RUN_MAX_BYTES - 1..text.len());
        assert!(lines[1].separator.is_empty());
        assert!(!lines[1].is_run_cap_break());
        assert!(lines
            .windows(2)
            .all(|lines| lines[0].source_range().end == lines[1].source_range().start));
    }

    #[test]
    fn hard_lines_keep_a_real_separator_on_the_last_capped_segment() {
        let text = format!("{}\r\nz", "a".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1));

        let lines = hard_lines(&text);

        assert_eq!(lines.len(), 3);
        assert!(lines[0].is_run_cap_break());
        assert!(lines[0].separator.is_empty());
        assert_eq!(
            lines[1].content,
            TEXT_SHAPING_RUN_MAX_BYTES..TEXT_SHAPING_RUN_MAX_BYTES + 1
        );
        assert_eq!(
            lines[1].separator,
            TEXT_SHAPING_RUN_MAX_BYTES + 1..TEXT_SHAPING_RUN_MAX_BYTES + 3
        );
        assert_eq!(lines[1].source_range().end, TEXT_SHAPING_RUN_MAX_BYTES + 3);
        assert_eq!(lines[2].content, TEXT_SHAPING_RUN_MAX_BYTES + 3..text.len());
        assert!(lines[2].separator.is_empty());
    }

    #[test]
    fn hard_line_count_includes_capped_runs() {
        let text = "a".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1);

        assert_eq!(hard_line_count(&text), 2);
    }

    #[test]
    fn hard_line_multiplicity_fast_path_matches_separators_and_run_cap() {
        assert!(!has_multiple_hard_lines("single line"));
        assert!(!has_multiple_hard_lines(
            &"a".repeat(TEXT_SHAPING_RUN_MAX_BYTES)
        ));
        assert!(has_multiple_hard_lines("first\r\nsecond"));
        assert!(has_multiple_hard_lines("first\u{2028}second"));
        assert!(has_multiple_hard_lines(
            &"a".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1)
        ));
    }

    #[test]
    fn hard_line_window_preserves_selected_unicode_separator_ranges() {
        let text = "a\r\nb\u{2028}c";

        let lines = hard_line_window(text, 1..3);

        assert_eq!(
            lines
                .iter()
                .map(|line| (line.content.clone(), line.separator.clone()))
                .collect::<Vec<_>>(),
            vec![(3..4, 4..7), (7..8, 8..8)]
        );
    }

    #[test]
    fn hard_line_window_selects_capped_segments_without_rewriting_separator() {
        let text = format!("{}\r\nz", "a".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1));

        let lines = hard_line_window(&text, 1..2);

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].content,
            TEXT_SHAPING_RUN_MAX_BYTES..TEXT_SHAPING_RUN_MAX_BYTES + 1
        );
        assert_eq!(
            lines[0].separator,
            TEXT_SHAPING_RUN_MAX_BYTES + 1..TEXT_SHAPING_RUN_MAX_BYTES + 3
        );
        assert!(!lines[0].is_run_cap_break());
    }

    #[test]
    fn hard_line_count_and_window_retains_the_document_count_without_full_materialization() {
        let text = format!(
            "a\r\nb\u{2028}{}",
            "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1)
        );

        let (count, lines) = hard_line_count_and_window(&text, 1..3);

        assert_eq!(count, 4);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].content, 3..4);
        assert_eq!(lines[0].separator, 4..7);
        assert_eq!(lines[1].content.start, 7);
        assert!(lines[1].is_run_cap_break());
    }

    #[test]
    fn hard_line_navigation_recognizes_every_canonical_separator() {
        let text = "ab\r\ncd\u{2028}ef\u{0085}gh\u{2029}ij\u{000b}kl\u{000c}mn\rop";
        let second_start = "ab\r\n".len();
        let third_start = "ab\r\ncd\u{2028}".len();
        let fourth_start = "ab\r\ncd\u{2028}ef\u{0085}".len();

        assert_eq!(hard_line_start(text, fourth_start + 1), fourth_start);
        assert_eq!(
            hard_line_end(text, second_start),
            third_start - "\u{2028}".len()
        );
        assert_eq!(
            next_hard_line_start(text, fourth_start - "\u{0085}".len()),
            Some(fourth_start)
        );
        assert_eq!(hard_line_start(text, second_start.saturating_sub(1)), 0);
        assert_eq!(hard_line_start(text, second_start), second_start);
        assert_eq!(
            hard_line_end(text, second_start.saturating_sub(1)),
            second_start - 2
        );
    }

    #[test]
    fn hard_line_visitor_preserves_capped_and_terminal_line_order() {
        let text = format!("{}\n", "a".repeat(TEXT_SHAPING_RUN_MAX_BYTES + 1));
        let mut visited = Vec::new();

        visit_hard_lines(&text, |line| visited.push(line));

        assert_eq!(visited.len(), 3);
        assert!(visited[0].is_run_cap_break());
        assert_eq!(
            visited[1].separator,
            TEXT_SHAPING_RUN_MAX_BYTES + 1..text.len()
        );
        assert_eq!(visited[2].content, text.len()..text.len());
    }
}
