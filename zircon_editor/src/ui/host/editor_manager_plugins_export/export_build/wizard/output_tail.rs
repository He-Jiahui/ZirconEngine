pub(super) const MAX_OUTPUT_TAIL_LINES: usize = 512;

const OUTPUT_TRUNCATION_MARKER: &str =
    "[earlier output truncated; full log is available as an artifact]";

pub(super) fn push_bounded_output_line(lines: &mut Vec<String>, line: String) -> u64 {
    if lines.len() < MAX_OUTPUT_TAIL_LINES {
        lines.push(line);
        return 0;
    }

    let dropped = if lines
        .first()
        .is_some_and(|value| value == OUTPUT_TRUNCATION_MARKER)
    {
        lines.remove(1);
        1
    } else {
        lines.drain(..2);
        lines.insert(0, OUTPUT_TRUNCATION_MARKER.to_string());
        2
    };
    lines.push(line);
    dropped
}

pub(super) fn retain_bounded_output_tail(lines: &mut Vec<String>) -> u64 {
    if lines.len() <= MAX_OUTPUT_TAIL_LINES {
        return 0;
    }

    let dropped = lines.len() - (MAX_OUTPUT_TAIL_LINES - 1);
    let retained = lines.split_off(dropped);
    lines.clear();
    lines.push(OUTPUT_TRUNCATION_MARKER.to_string());
    lines.extend(retained);
    dropped as u64
}

#[cfg(test)]
mod tests {
    use super::{
        push_bounded_output_line, retain_bounded_output_tail, MAX_OUTPUT_TAIL_LINES,
        OUTPUT_TRUNCATION_MARKER,
    };

    #[test]
    fn tail_never_exceeds_limit() {
        let mut lines = Vec::new();
        for index in 0..(MAX_OUTPUT_TAIL_LINES * 3) {
            push_bounded_output_line(&mut lines, format!("line-{index}"));
        }

        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(lines.last().map(String::as_str), Some("line-1535"));
    }

    #[test]
    fn truncation_marker_is_retained() {
        let mut lines = (0..(MAX_OUTPUT_TAIL_LINES + 10))
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>();
        let dropped = retain_bounded_output_tail(&mut lines);
        push_bounded_output_line(&mut lines, "last".to_string());

        assert_eq!(dropped, 11);
        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(
            lines.first().map(String::as_str),
            Some(OUTPUT_TRUNCATION_MARKER)
        );
        assert_eq!(lines.last().map(String::as_str), Some("last"));
    }
}
