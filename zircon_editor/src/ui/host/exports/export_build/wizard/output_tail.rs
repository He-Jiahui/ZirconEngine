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
    if lines.len() <= MAX_OUTPUT_TAIL_LINES {
        return 0;
    }

    let dropped = lines.len() - (MAX_OUTPUT_TAIL_LINES - 1);
    lines.drain(..dropped);
    lines.insert(0, OUTPUT_TRUNCATION_MARKER.to_string());
    dropped as u64
}

#[cfg(test)]
#[path = "tests/output_tail.rs"]
mod tests;
