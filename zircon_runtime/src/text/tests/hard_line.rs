use super::{
    hard_line_count, hard_line_count_and_window, hard_line_end, hard_line_start, hard_line_window,
    hard_lines, has_multiple_hard_lines, next_hard_line_start, visit_hard_lines,
};
use crate::text::TextShapingWorkBudget;

fn inline_threshold_bytes() -> usize {
    TextShapingWorkBudget::default().max_inline_input_bytes()
}

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
fn hard_lines_keep_an_oversized_unbroken_run_as_one_source_line() {
    let boundary = inline_threshold_bytes();
    let mut text = "a".repeat(boundary - 1);
    text.push('中');
    text.push('b');

    let lines = hard_lines(&text);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].content, 0..text.len());
    assert!(lines[0].separator.is_empty());
}

#[test]
fn hard_line_count_excludes_internal_shaping_chunks() {
    let text = "a".repeat(inline_threshold_bytes() + 1);

    assert_eq!(hard_line_count(&text), 1);
}

#[test]
fn hard_line_multiplicity_fast_path_matches_source_separators_only() {
    assert!(!has_multiple_hard_lines("single line"));
    assert!(!has_multiple_hard_lines(
        &"a".repeat(inline_threshold_bytes() + 1)
    ));
    assert!(has_multiple_hard_lines("first\r\nsecond"));
    assert!(has_multiple_hard_lines("first\u{2028}second"));
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
fn hard_line_window_never_selects_internal_shaping_chunks() {
    let boundary = inline_threshold_bytes();
    let text = format!("{}\r\nz", "a".repeat(boundary + 1));

    let lines = hard_line_window(&text, 1..2);

    assert_eq!(lines.len(), 1);
    assert!(lines[0].separator.is_empty());
    assert_eq!(lines[0].content, boundary + 3..boundary + 4);
}

#[test]
fn hard_line_count_and_window_retains_only_source_line_count() {
    let text = format!("a\r\nb\u{2028}{}", "x".repeat(inline_threshold_bytes() + 1));

    let (count, lines) = hard_line_count_and_window(&text, 1..3);

    assert_eq!(count, 3);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].content, 3..4);
    assert_eq!(lines[0].separator, 4..7);
    assert_eq!(lines[1].content.start, 7);
    assert!(lines[1].separator.is_empty());
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
fn hard_line_visitor_preserves_source_and_terminal_line_order() {
    let boundary = inline_threshold_bytes();
    let text = format!("{}\n", "a".repeat(boundary + 1));
    let mut visited = Vec::new();

    visit_hard_lines(&text, |line| visited.push(line));

    assert_eq!(visited.len(), 2);
    assert_eq!(visited[0].separator, boundary + 1..text.len());
    assert_eq!(visited[1].content, text.len()..text.len());
}
