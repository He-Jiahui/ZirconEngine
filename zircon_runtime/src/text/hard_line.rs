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
            '\n' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}' => true,
            _ => false,
        };
        if is_break {
            lines.push(HardLine {
                content: line_start..index,
                separator: index..next_start,
            });
            line_start = next_start;
        }
    }
    lines.push(HardLine {
        content: line_start..text.len(),
        separator: text.len()..text.len(),
    });
    lines
}

#[cfg(test)]
mod tests {
    use super::hard_lines;

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
}
