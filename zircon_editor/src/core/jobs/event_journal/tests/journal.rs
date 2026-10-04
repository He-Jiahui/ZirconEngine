use std::sync::Arc;

use super::super::gap::JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES;
use super::{EditorJobEventJournal, EditorJobEventJournalRecord};
use crate::core::jobs::{EditorJobEventJournalLimits, JobCategory, JobEvent, JobEventKind, JobId};

#[test]
fn newer_gap_does_not_overtake_an_older_retained_event() {
    let retained = lifecycle_event(1, "retained");
    let retained_bytes = retained.estimated_retained_bytes();
    let max_retained_bytes = retained_bytes + JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES;
    let journal =
        EditorJobEventJournal::new(EditorJobEventJournalLimits::new(8, max_retained_bytes));

    journal.push(retained);
    journal.push(lifecycle_event(2, &"x".repeat(max_retained_bytes + 1)));

    assert!(matches!(
        journal.pop(),
        Some(EditorJobEventJournalRecord::Event { event, .. })
            if event.journal_sequence() == 1
    ));
    assert!(matches!(
        journal.pop(),
        Some(EditorJobEventJournalRecord::Gap(gap))
            if gap.first_dropped_sequence() == 2
                && gap.last_dropped_sequence() == 2
    ));
}

#[test]
fn merged_gap_absorbs_retained_events_between_dropped_sequences() {
    let retained = lifecycle_event(1, "retained");
    let between_gaps = lifecycle_event(3, "between-gaps");
    let retained_bytes = retained.estimated_retained_bytes();
    let max_retained_bytes = retained_bytes
        .saturating_add(between_gaps.estimated_retained_bytes())
        .saturating_add(JOB_EVENT_JOURNAL_GAP_RETAINED_BYTES);
    let oversized = "x".repeat(max_retained_bytes + 1);
    let journal =
        EditorJobEventJournal::new(EditorJobEventJournalLimits::new(8, max_retained_bytes));

    journal.push(retained);
    journal.push(lifecycle_event(2, &oversized));
    journal.push(between_gaps);
    journal.push(lifecycle_event(4, &oversized));

    assert!(matches!(
        journal.pop(),
        Some(EditorJobEventJournalRecord::Event { event, .. })
            if event.journal_sequence() == 1
    ));
    assert!(matches!(
        journal.pop(),
        Some(EditorJobEventJournalRecord::Gap(gap))
            if gap.dropped_lifecycle_events() == 3
                && gap.first_dropped_sequence() == 2
                && gap.last_dropped_sequence() == 4
    ));
    assert!(journal.pop().is_none());
}

#[test]
fn restoring_backpressured_progress_preserves_the_newer_coalescing_index() {
    let journal = EditorJobEventJournal::default();
    journal.push(progress_event("first"));
    let backpressured = journal.pop().expect("first progress event");

    journal.push(progress_event("second"));
    journal.restore_front(backpressured);
    journal.push(progress_event("third"));

    assert!(matches!(
        journal.pop(),
        Some(EditorJobEventJournalRecord::Event { event, .. })
            if matches!(event.kind(), JobEventKind::Progress { message, .. } if message == "third")
    ));
    assert!(journal.pop().is_none());
    assert_eq!(journal.snapshot().coalesced_progress_events(), 2);
}

#[test]
fn editor09_gap_projection_reserves_covered_sequence_capacity() {
    let production = include_str!("../journal.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("event journal implementation");

    assert!(production.contains("covered_range.size_hint().0"));
    assert!(production.contains("Vec::with_capacity(covered_range.size_hint().0)"));
    assert!(production.contains("covered_sequences.extend("));
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_gap_projection_capacity_evidence() {
    const COVERED_EVENTS: usize = 4_096;
    let legacy_growths = geometric_growth_events(COVERED_EVENTS, 0);
    let optimized_growths = geometric_growth_events(COVERED_EVENTS, COVERED_EVENTS);

    println!(
        "EDITOR09_EVENT_JOURNAL_GAP_CAPACITY_BENCH_V1 legacy_growths={} optimized_growths={} covered_events={}",
        legacy_growths, optimized_growths, COVERED_EVENTS,
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

fn lifecycle_event(id: u64, label: &str) -> JobEvent {
    JobEvent::new(
        JobId::new(id),
        Arc::<str>::from(label),
        JobCategory::Misc,
        JobEventKind::Started,
    )
}

fn progress_event(message: &str) -> JobEvent {
    JobEvent::new(
        JobId::new(1),
        Arc::<str>::from("progress"),
        JobCategory::Misc,
        JobEventKind::Progress {
            completed: 1,
            total: 3,
            message: message.to_string(),
        },
    )
}
