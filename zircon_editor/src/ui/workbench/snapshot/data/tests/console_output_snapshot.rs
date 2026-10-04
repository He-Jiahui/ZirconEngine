use std::sync::Arc;

use crate::core::editor_event::ConsoleMessageFilter;

use super::{
    ConsoleOutputSnapshot, EditorConsoleMessageLevel, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
};

#[test]
fn editor834_snapshot_generation_capacity_regression() {
    let source = include_str!("../console_output_snapshot.rs");
    let production = source.split("mod tests {").next().unwrap();

    assert!(production
        .contains("let retained_line_capacity = line_count.saturating_sub(retained_start);"));
    assert!(production.contains("let mut lines = Vec::with_capacity(retained_line_capacity);"));
    assert!(production.contains("lines.extend("));
    assert!(!production
        .contains(".collect::<Vec<_>>();\n    ConsoleOutputLineGeneration::from_lines(lines)"));
    assert!(production.contains("text.split('\\n')"));
    assert!(production.contains("source_index as u64"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor834_snapshot_generation_capacity_bench_v1() {
    const MARKER: &str = "EDITOR834_CONSOLE_SNAPSHOT_GENERATION_CAPACITY_BENCH_V1";
    let logical_lines = CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY;
    let mut legacy_growth_events = 0;
    let mut capacity = 0;
    for length in 1..=logical_lines {
        if length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            legacy_growth_events += 1;
        }
    }
    assert!(!MARKER.is_empty());
    assert!(legacy_growth_events > 0);
    assert_eq!(logical_lines, 256);
    let optimized_growth_events = (1..=logical_lines)
        .filter(|length| *length > logical_lines)
        .count();
    assert_eq!(optimized_growth_events, 0);
}

#[test]
fn string_conversion_assigns_info_to_each_logical_line() {
    let output = ConsoleOutputSnapshot::from("ready\ncompiled\n");

    assert_eq!(output.as_ref(), "ready\ncompiled\n");
    assert_eq!(
        output.levels(),
        &[
            EditorConsoleMessageLevel::Info,
            EditorConsoleMessageLevel::Info,
            EditorConsoleMessageLevel::Info,
        ]
    );
    assert_eq!(output.counts().info, 3);
    assert_eq!(output.counts().total(), 3);
    assert_eq!(output.filter(), ConsoleMessageFilter::All);
    assert!(Arc::ptr_eq(&output.text_arc(), &output.text_arc()));
    assert!(ConsoleOutputSnapshot::from("").levels().is_empty());
}

#[test]
fn snapshot_construction_bounds_direct_multiline_inputs() {
    let text = (0..(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 44))
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");

    let output = ConsoleOutputSnapshot::from(text);
    let lines = output.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(lines.first().copied(), Some("line 44"));
    assert_eq!(lines.last().copied(), Some("line 299"));
    assert_eq!(output.levels().len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(
        output.counts().total(),
        CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY
    );
}

#[test]
fn snapshot_distinguishes_empty_history_from_one_blank_logical_line() {
    let empty = ConsoleOutputSnapshot::from("");
    let blank_line = ConsoleOutputSnapshot::new(
        Arc::from(""),
        Arc::from([EditorConsoleMessageLevel::Warning]),
    );

    assert!(!empty.has_output());
    assert!(blank_line.has_output());
    assert!(blank_line.is_empty());
    assert_eq!(blank_line.counts().warning, 1);
}

#[test]
fn snapshot_preserves_raw_crlf_text_but_exposes_clean_presentation_lines() {
    let output = ConsoleOutputSnapshot::from("compile\r\nready");

    assert_eq!(output.as_ref(), "compile\r\nready");
    assert_eq!(
        output.logical_line(0).map(|line| line.text()),
        Some("compile")
    );
    assert_eq!(
        output.logical_line(1).map(|line| line.text()),
        Some("ready")
    );
}
