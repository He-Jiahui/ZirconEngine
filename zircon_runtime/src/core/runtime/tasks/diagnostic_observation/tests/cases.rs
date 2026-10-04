use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::{
    TaskDiagnosticJournal, TaskDiagnosticKind, TaskDiagnosticSeverity,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES, TASK_DIAGNOSTIC_RETENTION_CAPACITY,
};

const PARTITION_SAMPLE_COUNT: usize = 17;
const PARTITION_ITERATIONS: usize = 20_000;

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

#[test]
fn optimization_batch_hi_runtime596_late_cursor_preserves_page_boundaries() {
    let journal = Arc::new(TaskDiagnosticJournal::default());
    for index in 0..32 {
        journal.record(
            super::TaskDiagnosticIdentity::new(journal.source_id(), index + 1),
            TaskDiagnosticKind::Cancelled,
            Arc::from(format!("late-cursor-{index}")),
        );
    }

    let cursor = super::TaskDiagnosticCursor::new(journal.source_id(), 25);
    let batch = journal.read_after(cursor, 4);
    let sequences = batch
        .observations()
        .iter()
        .map(super::TaskDiagnosticObservation::observation_sequence)
        .collect::<Vec<_>>();

    assert_eq!(sequences, vec![25, 26, 27, 28]);
    assert_eq!(batch.recovery_cursor().next_observation_sequence(), 25);
    assert_eq!(batch.next_cursor().next_observation_sequence(), 29);
    assert!(batch.has_more());
}

#[test]
fn optimization_batch_hi_runtime596_zero_limit_preserves_cursor_without_a_page() {
    let journal = Arc::new(TaskDiagnosticJournal::default());
    journal.record(
        super::TaskDiagnosticIdentity::new(journal.source_id(), 1),
        TaskDiagnosticKind::Cancelled,
        Arc::from("zero-limit"),
    );

    let cursor = journal.initial_cursor();
    let batch = journal.read_after(cursor, 0);

    assert!(batch.observations().is_empty());
    assert_eq!(batch.recovery_cursor(), cursor);
    assert_eq!(batch.next_cursor(), cursor);
    assert!(batch.has_more());
}

#[test]
fn optimization_batch_hi_runtime596_journal_uses_ordered_partition() {
    let production = include_str!("../journal.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();

    assert!(production.contains(".partition_point(|entry|"));
    assert!(production.contains("if read_limit == 0"));
    assert!(!production.contains(".filter(|entry| entry.observation_sequence()"));
}

#[test]
fn optimization_batch_runtime02_task_diagnostic_page_reserves_result_capacity() {
    let production = include_str!("../journal.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();

    assert!(production.contains("let page_capacity = read_limit.min("));
    assert!(production.contains("Vec::with_capacity(page_capacity)"));
    assert!(production.contains("observations.extend("));
    assert!(production.contains("if read_limit == 0"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_runtime02_task_diagnostic_page_capacity_bench() {
    let page_entries = TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES;
    let legacy_growths = geometric_growth_events(page_entries, 0);
    let optimized_growths = geometric_growth_events(page_entries, page_entries);

    println!(
        "RUNTIME02_TASK_DIAGNOSTIC_PAGE_CAPACITY_BENCH_V1 legacy_growths={} optimized_growths={} page_entries={}",
        legacy_growths, optimized_growths, page_entries,
    );
    assert!(legacy_growths > 0);
    assert_eq!(optimized_growths, 0);
}

fn geometric_growth_events(target_len: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut growths = 0;
    for length in 0..target_len {
        if length == capacity {
            capacity = capacity.max(1).saturating_mul(2);
            growths += 1;
        }
    }
    growths
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_runtime596_ordered_partition_bench() {
    let sequences = (1..=TASK_DIAGNOSTIC_RETENTION_CAPACITY as u64).collect::<VecDeque<_>>();
    let recovered_sequence = TASK_DIAGNOSTIC_RETENTION_CAPACITY as u64 - 15;
    let mut legacy_samples = Vec::with_capacity(PARTITION_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(PARTITION_SAMPLE_COUNT);
    for sample in 0..PARTITION_SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..PARTITION_ITERATIONS {
                black_box(
                    black_box(&sequences)
                        .iter()
                        .position(|sequence| *sequence >= black_box(recovered_sequence)),
                );
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..PARTITION_ITERATIONS {
                black_box(
                    black_box(&sequences)
                        .partition_point(|sequence| *sequence < black_box(recovered_sequence)),
                );
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let p95_index = (PARTITION_SAMPLE_COUNT * 95).div_ceil(100) - 1;
    let legacy_p95 = legacy_samples[p95_index];
    let optimized_p95 = optimized_samples[p95_index];
    println!(
        "RUNTIME596_TASK_DIAGNOSTIC_PARTITION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} retained_entries={} cursor_from_tail=16",
        legacy_p95,
        optimized_p95,
        PARTITION_SAMPLE_COUNT,
        PARTITION_ITERATIONS,
        TASK_DIAGNOSTIC_RETENTION_CAPACITY,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(50),
        "ordered partition P95 must be at most 50% of the linear prefix scan P95"
    );
}
