use std::sync::Arc;

use super::{
    TaskDiagnosticJournal, TaskDiagnosticKind, TaskDiagnosticSeverity,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES, TASK_DIAGNOSTIC_RETENTION_CAPACITY,
};

#[test]
fn journal_retention_is_bounded_and_reports_the_exact_cursor_gap() {
    let journal = Arc::new(TaskDiagnosticJournal::default());
    let cursor = journal.initial_cursor();

    for index in 0..TASK_DIAGNOSTIC_RETENTION_CAPACITY + 3 {
        let identity = super::TaskDiagnosticIdentity::new(journal.source_id(), index as u64 + 1);
        journal.record(
            identity,
            TaskDiagnosticKind::Cancelled,
            Arc::from(format!("cancelled-{index}")),
        );
    }

    let mut batch = journal.read_after(cursor, TASK_DIAGNOSTIC_RETENTION_CAPACITY + 10);
    assert_eq!(batch.dropped_count(), 3);
    assert_eq!(
        batch.observations().len(),
        TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES
    );
    assert!(batch.has_more());
    assert_eq!(batch.recovery_cursor().next_observation_sequence(), 4);
    assert_eq!(
        batch.next_cursor().next_observation_sequence(),
        4 + TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES as u64
    );
    assert_eq!(
        batch.observations()[0].message(),
        "cancelled-3",
        "the retained suffix must begin immediately after the reported gap"
    );
    let mut retained_count = batch.observations().len();
    while batch.has_more() {
        batch = journal.read_after(batch.next_cursor(), TASK_DIAGNOSTIC_RETENTION_CAPACITY + 10);
        assert_eq!(batch.dropped_count(), 0);
        assert!(batch.observations().len() <= TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES);
        retained_count += batch.observations().len();
    }
    assert_eq!(retained_count, TASK_DIAGNOSTIC_RETENTION_CAPACITY);
}

#[test]
fn terminal_kinds_preserve_runtime_neutral_severity_and_task_identity() {
    let journal = Arc::new(TaskDiagnosticJournal::default());
    let cursor = journal.initial_cursor();
    let cancelled = super::TaskDiagnosticIdentity::new(journal.source_id(), 1);
    let panicked = super::TaskDiagnosticIdentity::new(journal.source_id(), 2);

    journal.record(
        cancelled,
        TaskDiagnosticKind::Cancelled,
        Arc::from("cancelled before launch"),
    );
    journal.record(
        panicked,
        TaskDiagnosticKind::Panicked,
        Arc::from("worker panic"),
    );

    let batch = journal.read_after(cursor, 8);
    let observations = batch.observations();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].identity(), cancelled);
    assert_eq!(observations[0].severity(), TaskDiagnosticSeverity::Warning);
    assert_eq!(observations[1].identity(), panicked);
    assert_eq!(observations[1].severity(), TaskDiagnosticSeverity::Error);
}

#[test]
fn observation_messages_are_utf8_safely_bounded() {
    let journal = Arc::new(TaskDiagnosticJournal::default());
    let cursor = journal.initial_cursor();
    let message = "任".repeat(super::MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES);

    journal.record(
        super::TaskDiagnosticIdentity::new(journal.source_id(), 1),
        TaskDiagnosticKind::Panicked,
        Arc::from(message),
    );

    let batch = journal.read_after(cursor, 1);
    let retained = batch.observations()[0].message();
    assert!(retained.len() <= super::MAX_TASK_DIAGNOSTIC_MESSAGE_BYTES);
    assert!(retained.is_char_boundary(retained.len()));
}
