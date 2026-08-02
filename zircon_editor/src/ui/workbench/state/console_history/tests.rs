use super::EditorConsoleHistory;
use crate::core::editor_event::ConsoleMessageFilter;
use crate::ui::workbench::snapshot::{
    CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY, EditorConsoleMessageLevel,
};
use std::sync::Arc;

#[test]
fn console_history_is_bounded_and_preserves_latest_message_order() {
    let mut history = EditorConsoleHistory::new("Ready");

    for index in 0..(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 4) {
        history.push(&format!("message {index}"));
    }

    let output = history.output();
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(lines.first().copied(), Some("message 4"));
    let expected_last = format!("message {}", CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 3);
    assert_eq!(lines.last().copied(), Some(expected_last.as_str()));
}

#[test]
fn console_history_bounds_one_multiline_message_by_logical_lines() {
    let mut history = EditorConsoleHistory::new("");
    let message = (0..(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 44))
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");

    history.push_with_level(&message, EditorConsoleMessageLevel::Warning);

    let output = history.output();
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(lines.first().copied(), Some("line 44"));
    assert_eq!(lines.last().copied(), Some("line 299"));
    assert_eq!(output.levels().len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(
        output.counts().warning,
        CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY
    );
}

#[test]
fn console_history_trims_only_the_oldest_logical_lines_across_messages() {
    let mut history = EditorConsoleHistory::new("");
    let old_message = (0..200)
        .map(|index| format!("old {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let new_message = (0..100)
        .map(|index| format!("new {index}"))
        .collect::<Vec<_>>()
        .join("\n");

    history.push_with_level(&old_message, EditorConsoleMessageLevel::Info);
    history.push_with_level(&new_message, EditorConsoleMessageLevel::Error);

    let output = history.output();
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(lines.first().copied(), Some("old 44"));
    assert_eq!(lines.last().copied(), Some("new 99"));
    assert_eq!(output.counts().info, 156);
    assert_eq!(output.counts().error, 100);
}

#[test]
fn console_history_preserves_a_retained_empty_line_and_its_level() {
    let mut history = EditorConsoleHistory::new("");
    history.push_with_level("visible\n", EditorConsoleMessageLevel::Info);
    assert!(history.set_filter(ConsoleMessageFilter::Info));
    let hidden_message = (0..(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY - 1))
        .map(|index| format!("hidden {index}"))
        .collect::<Vec<_>>()
        .join("\n");

    history.push_with_level(&hidden_message, EditorConsoleMessageLevel::Error);

    let output = history.output();
    assert!(output.is_empty());
    assert!(output.has_output());
    assert_eq!(output.levels(), &[EditorConsoleMessageLevel::Info]);
    assert_eq!(output.counts().info, 1);
    assert_eq!(
        output.counts().error,
        CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY - 1
    );
}

#[test]
fn console_history_skips_blank_and_consecutive_duplicate_messages() {
    let mut history = EditorConsoleHistory::new("Ready");
    let initial = history.output();

    history.push("  ");
    history.push("Ready");

    assert_eq!(initial.as_ptr(), history.output().as_ptr());
}

#[test]
fn console_history_clear_is_idempotent_and_accepts_the_current_status_again() {
    let mut history = EditorConsoleHistory::new("Ready");

    assert!(history.clear());
    assert!(history.output().is_empty());
    assert!(!history.clear());

    history.push("Ready");
    assert_eq!(history.output().as_ref(), "Ready");
}

#[test]
fn console_history_keeps_a_level_for_every_logical_output_line() {
    let mut history = EditorConsoleHistory::new("");

    history.push_with_level(
        "compile warning\ndetail",
        EditorConsoleMessageLevel::Warning,
    );
    history.push_with_level("compile warning\ndetail", EditorConsoleMessageLevel::Error);

    let output = history.output();
    assert_eq!(
        output.as_ref(),
        "compile warning\ndetail\ncompile warning\ndetail"
    );
    assert_eq!(
        output.levels(),
        &[
            EditorConsoleMessageLevel::Warning,
            EditorConsoleMessageLevel::Warning,
            EditorConsoleMessageLevel::Error,
            EditorConsoleMessageLevel::Error,
        ]
    );
}

#[test]
fn console_history_filter_changes_visible_output_without_changing_total_counts() {
    let mut history = EditorConsoleHistory::new("");
    history.push_with_level("ready", EditorConsoleMessageLevel::Info);
    history.push_with_level(
        "compile warning\ndetail",
        EditorConsoleMessageLevel::Warning,
    );
    history.push_with_level("pipeline failed", EditorConsoleMessageLevel::Error);

    assert!(history.set_filter(ConsoleMessageFilter::Warning));
    let warning_output = history.output();
    assert_eq!(warning_output.as_ref(), "compile warning\ndetail");
    assert_eq!(warning_output.filter(), ConsoleMessageFilter::Warning);
    assert_eq!(warning_output.counts().info, 1);
    assert_eq!(warning_output.counts().warning, 2);
    assert_eq!(warning_output.counts().error, 1);

    let text = warning_output.text_arc();
    assert!(!history.set_filter(ConsoleMessageFilter::Warning));
    assert!(Arc::ptr_eq(&text, &history.output().text_arc()));

    assert!(history.set_filter(ConsoleMessageFilter::All));
    assert_eq!(
        history.output().as_ref(),
        "ready\ncompile warning\ndetail\npipeline failed"
    );
}

#[test]
fn console_history_clear_preserves_filter_for_subsequent_messages() {
    let mut history = EditorConsoleHistory::new("ready");
    assert!(history.set_filter(ConsoleMessageFilter::Error));
    assert!(history.clear());
    assert_eq!(history.output().filter(), ConsoleMessageFilter::Error);

    history.push_with_level("still ready", EditorConsoleMessageLevel::Info);
    assert!(history.output().is_empty());
    assert_eq!(history.output().counts().info, 1);

    history.push_with_level("pipeline failed", EditorConsoleMessageLevel::Error);
    assert_eq!(history.output().as_ref(), "pipeline failed");
    assert_eq!(history.output().counts().error, 1);
}
