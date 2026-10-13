use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::editor_message::DocumentId;

use super::*;

const CHANGE_COUNT: usize = 65_536;
const UNIQUE_HISTORY_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn document(value: u64) -> HistoryContextId {
    HistoryContextId::Document(DocumentId::new(value))
}

// Retain V1's historical estimator and numerical gate without reinterpreting old receipts.
fn legacy_percentile_95_estimate(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    assert!(!samples.is_empty());
    samples.sort_unstable();
    // ceil(0.95 * n) == n - floor(n / 20); subtract one for the zero-based rank.
    samples[samples.len() - samples.len() / 20 - 1]
}

fn legacy_changed_histories(changes: &[u64]) -> Vec<u64> {
    changes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn optimized_changed_histories(changes: &[u64]) -> Vec<u64> {
    let mut changed = HashSet::with_capacity(changes.len());
    changed.extend(changes.iter().copied());
    let mut changed = changed.into_iter().collect::<Vec<_>>();
    changed.sort_unstable();
    changed
}

fn legacy_bounded_histories(changes: &[HistoryContextId]) -> Vec<Vec<HistoryContextId>> {
    changes
        .chunks(HISTORY_DIRTY_JOURNAL_CAPACITY)
        .map(|changes| {
            changes
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        })
        .collect()
}

fn dirty_journals(changes: &[HistoryContextId]) -> Vec<HistoryDirtyJournal> {
    changes
        .chunks(HISTORY_DIRTY_JOURNAL_CAPACITY)
        .map(|changes| {
            let mut journal = HistoryDirtyJournal::default();
            for history in changes {
                let reservation = journal.reserve_dirty_change().unwrap();
                journal.record_dirty_change(*history, reservation);
            }
            journal
        })
        .collect()
}

fn materialize_bounded_histories(
    journals: &mut [HistoryDirtyJournal],
) -> Vec<Vec<HistoryContextId>> {
    journals
        .iter_mut()
        .map(|journal| journal.changed_histories_after(0))
        .collect()
}

#[test]
fn optimization_batch_20260826o_editor63_hash_dedup_preserves_sorted_dirty_delta() {
    let mut journal = HistoryDirtyJournal::default();
    for history in [document(3), document(1), document(3), document(2)] {
        let reservation = journal.reserve_dirty_change().unwrap();
        journal.record_dirty_change(history, reservation);
    }

    let changed: Vec<HistoryContextId> = journal.changed_histories_after(0);

    assert_eq!(changed, vec![document(1), document(2), document(3)]);
    assert_eq!(journal.journal_visits, 4);
}

#[test]
fn optimization_batch_20260826o_editor63_dirty_journal_uses_hash_dedup() {
    let source = include_str!("../dirty_batch.rs");
    let production = source
        .split("#[cfg(test)]\nmod optimization_tests")
        .next()
        .unwrap();

    assert!(production.contains("HashSet::with_capacity"));
    assert!(production.contains("changed.sort_unstable();"));
    assert!(production.contains("-> Vec<HistoryContextId>"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
fn nearest_rank_p95_uses_the_correct_sample_for_small_and_exact_ranks() {
    for count in [1, 17, 20, 100] {
        let mut samples = (1..=count)
            .rev()
            .map(Duration::from_nanos)
            .collect::<Vec<_>>();
        assert_eq!(
            percentile_95(&mut samples),
            Duration::from_nanos(count - count / 20)
        );
    }
    let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();
    assert_eq!(
        legacy_percentile_95_estimate(&mut samples),
        Duration::from_nanos(16)
    );
    assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826o_editor63_dirty_journal_hash_dedup_performance_evidence() {
    let changes = (0..CHANGE_COUNT)
        .map(|index| ((index * 4_099) % UNIQUE_HISTORY_COUNT) as u64)
        .collect::<Vec<_>>();
    let expected = (0..UNIQUE_HISTORY_COUNT as u64).collect::<Vec<_>>();
    assert_eq!(legacy_changed_histories(&changes), expected);
    assert_eq!(optimized_changed_histories(&changes), expected);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_changed_histories(black_box(&changes)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(optimized_changed_histories(black_box(&changes)));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(optimized_changed_histories(black_box(&changes)));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_changed_histories(black_box(&changes)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p95 = legacy_percentile_95_estimate(&mut legacy_samples);
    let optimized_p95 = legacy_percentile_95_estimate(&mut optimized_samples);
    println!(
        "EDITOR63_DIRTY_JOURNAL_HASH_DEDUP_BENCH_V1 changes={CHANGE_COUNT} \
             unique_histories={UNIQUE_HISTORY_COUNT} ordered_admissions={CHANGE_COUNT} \
             hash_admissions={CHANGE_COUNT} sorted_values={UNIQUE_HISTORY_COUNT} \
             producer=flat_u64_algorithm_model percentile_estimator=legacy_floor_v1 \
             legacy_p95_ns={} optimized_p95_ns={}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
        "V1 hash-dedup historical estimate {:?} exceeded 60% of ordered-dedup estimate {:?}",
        optimized_p95,
        legacy_p95,
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20261013_editor63_bounded_dirty_journal_materialization_performance_evidence_v2(
) {
    let changes = (0..CHANGE_COUNT)
        .map(|index| document(((index * 4_099) % UNIQUE_HISTORY_COUNT) as u64))
        .collect::<Vec<_>>();
    // These are independent prepared windows, not one sequential journal lifecycle.
    // Recording and fixture preparation remain outside both materialization timers.
    let expected = changes
        .chunks(HISTORY_DIRTY_JOURNAL_CAPACITY)
        .map(|changes| {
            let mut expected = changes.to_vec();
            expected.sort_unstable();
            expected.dedup();
            expected
        })
        .collect::<Vec<_>>();
    let mut journals = dirty_journals(&changes);
    assert_eq!(
        expected
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        UNIQUE_HISTORY_COUNT
    );
    assert!(journals
        .iter()
        .all(|journal| journal.changes.len() == HISTORY_DIRTY_JOURNAL_CAPACITY));
    assert_eq!(
        journals
            .iter()
            .map(|journal| journal.changes.len())
            .sum::<usize>(),
        CHANGE_COUNT
    );
    assert_eq!(legacy_bounded_histories(&changes), expected);
    assert_eq!(materialize_bounded_histories(&mut journals), expected);
    assert_eq!(
        journals
            .iter()
            .map(|journal| journal.journal_visits)
            .sum::<usize>(),
        CHANGE_COUNT
    );
    for journal in &mut journals {
        journal.journal_visits = 0;
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_bounded_histories(black_box(&changes)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(materialize_bounded_histories(black_box(&mut journals)));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(materialize_bounded_histories(black_box(&mut journals)));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_bounded_histories(black_box(&changes)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let sorted_values = expected.iter().map(Vec::len).sum::<usize>();
    assert_eq!(sorted_values, CHANGE_COUNT);
    assert_eq!(
        journals
            .iter()
            .map(|journal| journal.journal_visits)
            .sum::<usize>(),
        CHANGE_COUNT * SAMPLE_COUNT
    );
    println!(
        "EDITOR63_BOUNDED_DIRTY_JOURNAL_MATERIALIZATION_BENCH_V2 changes={CHANGE_COUNT} \
             unique_histories={UNIQUE_HISTORY_COUNT} ordered_admissions={CHANGE_COUNT} \
             hash_admissions={CHANGE_COUNT} sorted_values={sorted_values} \
             journal_capacity={HISTORY_DIRTY_JOURNAL_CAPACITY} journal_windows={} \
             producer=HistoryDirtyJournal::changed_histories_after \
             timed_phase=materialization percentile_estimator=nearest_rank samples={SAMPLE_COUNT} \
             legacy_p95_ns={} optimized_p95_ns={}",
        journals.len(),
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
        "V2 materialization P95 {:?} exceeded 60% of bounded ordered-dedup P95 {:?}",
        optimized_p95,
        legacy_p95,
    );
}
