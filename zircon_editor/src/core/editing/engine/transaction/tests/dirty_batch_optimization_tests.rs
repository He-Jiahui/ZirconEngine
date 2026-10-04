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

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
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

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR63_DIRTY_JOURNAL_HASH_DEDUP_BENCH_V1 changes={CHANGE_COUNT} \
             unique_histories={UNIQUE_HISTORY_COUNT} ordered_admissions={CHANGE_COUNT} \
             hash_admissions={CHANGE_COUNT} sorted_values={UNIQUE_HISTORY_COUNT} \
             legacy_p95_ns={} optimized_p95_ns={}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
        "hash-dedup P95 {:?} exceeded 60% of ordered-dedup P95 {:?}",
        optimized_p95,
        legacy_p95,
    );
}
