use std::collections::VecDeque;

pub(super) const MAX_OUTPUT_TAIL_LINES: usize = 512;

const OUTPUT_TRUNCATION_MARKER: &str =
    "[earlier output truncated; full log is available as an artifact]";

pub(super) fn push_bounded_output_line(lines: &mut VecDeque<String>, line: String) -> u64 {
    if lines.len() < MAX_OUTPUT_TAIL_LINES {
        lines.push_back(line);
        return 0;
    }

    let dropped = if lines
        .front()
        .is_some_and(|value| value == OUTPUT_TRUNCATION_MARKER)
    {
        let marker = lines
            .pop_front()
            .expect("a marked output tail must retain its marker");
        let _ = lines.pop_front();
        lines.push_front(marker);
        1
    } else {
        let _ = lines.pop_front();
        let _ = lines.pop_front();
        lines.push_front(OUTPUT_TRUNCATION_MARKER.to_string());
        2
    };
    lines.push_back(line);
    dropped
}

pub(super) fn retain_bounded_output_tail(lines: &mut VecDeque<String>) -> u64 {
    if lines.len() <= MAX_OUTPUT_TAIL_LINES {
        return 0;
    }

    let dropped = lines.len() - (MAX_OUTPUT_TAIL_LINES - 1);
    for _ in 0..dropped {
        let _ = lines.pop_front();
    }
    lines.push_front(OUTPUT_TRUNCATION_MARKER.to_string());
    dropped as u64
}

pub(super) fn retain_bounded_output_lines(lines: &mut Vec<String>) -> u64 {
    let mut tail = std::mem::take(lines).into_iter().collect::<VecDeque<_>>();
    let dropped = retain_bounded_output_tail(&mut tail);
    *lines = tail.into_iter().collect();
    dropped
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{
        push_bounded_output_line, retain_bounded_output_lines, retain_bounded_output_tail,
        MAX_OUTPUT_TAIL_LINES, OUTPUT_TRUNCATION_MARKER,
    };

    #[test]
    fn tail_never_exceeds_limit() {
        let mut lines = VecDeque::new();
        for index in 0..(MAX_OUTPUT_TAIL_LINES * 3) {
            push_bounded_output_line(&mut lines, format!("line-{index}"));
        }

        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(lines.back().map(String::as_str), Some("line-1535"));
    }

    #[test]
    fn truncation_marker_is_retained() {
        let mut lines = (0..(MAX_OUTPUT_TAIL_LINES + 10))
            .map(|index| format!("line-{index}"))
            .collect::<VecDeque<_>>();
        let dropped = retain_bounded_output_tail(&mut lines);
        push_bounded_output_line(&mut lines, "last".to_string());

        assert_eq!(dropped, 11);
        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(
            lines.front().map(String::as_str),
            Some(OUTPUT_TRUNCATION_MARKER)
        );
        assert_eq!(lines.back().map(String::as_str), Some("last"));
    }

    #[test]
    fn terminal_vec_results_are_bounded_at_the_output_boundary() {
        let mut lines = (0..(MAX_OUTPUT_TAIL_LINES + 10))
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>();

        let dropped = retain_bounded_output_lines(&mut lines);

        assert_eq!(dropped, 11);
        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(
            lines.first().map(String::as_str),
            Some(OUTPUT_TRUNCATION_MARKER)
        );
        assert_eq!(lines.last().map(String::as_str), Some("line-521"));
    }
}
