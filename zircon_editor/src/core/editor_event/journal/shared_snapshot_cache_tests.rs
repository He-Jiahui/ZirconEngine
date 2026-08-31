use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use serde_json::json;

use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventId, EditorEventResult, EditorEventRetentionPolicy,
    EditorEventSequence, EditorEventSource, EditorEventUndoPolicy, EditorOperationEvent,
};

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hq_editor_reuses_unchanged_journal_record_storage() {
    let mut store = EditorEventJournalStore::new(EditorEventRetentionPolicy::default().journal);
    store.push(shared_record(1, 32));

    let first = store.snapshot();
    let second = store.snapshot();

    assert_eq!(first.records(), second.records());
    assert!(Arc::ptr_eq(&first.records, &second.records));
}

#[test]
fn optimization_batch_20260828hq_editor_refreshes_cache_after_a_new_event() {
    let mut store = EditorEventJournalStore::new(EditorEventRetentionPolicy::default().journal);
    store.push(shared_record(1, 32));
    let before = store.snapshot();

    store.push(shared_record(2, 32));
    let changed = store.snapshot();
    let stable = store.snapshot();

    assert_eq!(changed.records().len(), 2);
    assert!(!Arc::ptr_eq(&before.records, &changed.records));
    assert!(Arc::ptr_eq(&changed.records, &stable.records));
    assert_eq!(
        serde_json::to_value(&changed).unwrap()["records"][1]["sequence"],
        2
    );
}

#[test]
fn optimization_batch_hi_editor591_snapshot_cache_tracks_retention_generation() {
    let mut store = EditorEventJournalStore::new(EditorEventRetentionPolicy::default().journal);
    store.push(shared_record(1, 32));
    let first = store.snapshot();
    let generation = store.records.generation();
    let stable = store.snapshot();

    assert_eq!(generation, store.records.generation());
    assert!(Arc::ptr_eq(&first.records, &stable.records));

    store.records.acknowledge_through_delivery_cursor(1);
    let after_acknowledge = store.snapshot();
    assert!(after_acknowledge.records.is_empty());
    assert!(!Arc::ptr_eq(&first.records, &after_acknowledge.records));
}

#[test]
fn optimization_batch_hi_editor591_snapshot_cache_does_not_scan_shared_records_when_stable() {
    let source = include_str!("../journal.rs");
    assert!(source.contains("generation_after_prune"));
    assert!(!source.contains("Arc::ptr_eq(current, cached)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hq_editor_shared_event_journal_snapshot_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 32;
    let mut store = EditorEventJournalStore::new(EditorEventRetentionPolicy::default().journal);
    for sequence in 1..=512 {
        store.push(shared_record(sequence, 8 * 1024));
    }

    black_box(store.snapshot());
    black_box(legacy_snapshot(&mut store));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = |store: &mut EditorEventJournalStore| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_snapshot(black_box(&mut *store)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = |store: &mut EditorEventJournalStore| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(black_box(&mut *store).snapshot());
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy(&mut store));
            optimized_samples.push(measure_optimized(&mut store));
        } else {
            optimized_samples.push(measure_optimized(&mut store));
            legacy_samples.push(measure_legacy(&mut store));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR209_SHARED_EVENT_JOURNAL_SNAPSHOT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_editor591_journal_generation_snapshot_benchmark() {
    const SAMPLES: usize = 17;
    const ITERATIONS: usize = 256;
    let mut store = EditorEventJournalStore::new(EditorEventRetentionPolicy::default().journal);
    for sequence in 1..=4_096 {
        store.push(shared_record(sequence, 8));
    }
    black_box(store.snapshot());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy_generation_scan(&mut store, ITERATIONS));
            optimized_samples.push(measure_optimized_snapshot(&mut store, ITERATIONS));
        } else {
            optimized_samples.push(measure_optimized_snapshot(&mut store, ITERATIONS));
            legacy_samples.push(measure_legacy_generation_scan(&mut store, ITERATIONS));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR591_JOURNAL_GENERATION_BENCH_V1 sample_pairs={SAMPLES} iterations={ITERATIONS} retained_records=4096 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn measure_legacy_generation_scan(store: &mut EditorEventJournalStore, iterations: usize) -> u128 {
    let cached_shared_records = store.records.records();
    let started = Instant::now();
    for _ in 0..iterations {
        let shared_records = store.records.records();
        let cache_is_current = shared_records.len() == cached_shared_records.len()
            && shared_records
                .iter()
                .zip(&cached_shared_records)
                .all(|(current, cached)| Arc::ptr_eq(current, cached));
        black_box(cache_is_current);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized_snapshot(store: &mut EditorEventJournalStore, iterations: usize) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(store.snapshot());
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_snapshot(store: &mut EditorEventJournalStore) -> EditorEventJournal {
    let records = store
        .records
        .records()
        .into_iter()
        .map(|record| record.record().clone())
        .collect::<Vec<_>>()
        .into();
    EditorEventJournal {
        records,
        retention_diagnostics: store.records.diagnostics(),
        retention_budgets: store.records.budgets(),
    }
}

fn shared_record(sequence: u64, payload_bytes: usize) -> Arc<SharedEditorEventRecord> {
    Arc::new(SharedEditorEventRecord::new(EditorEventRecord {
        event_id: EditorEventId::new(sequence),
        sequence: EditorEventSequence::new(sequence),
        source: EditorEventSource::Headless,
        event: EditorEvent::Operation(EditorOperationEvent::CommandExecuted {
            operation_id: format!("benchmark.operation.{sequence}"),
            transaction_id: sequence,
            group_open: false,
        }),
        binding_path: None,
        operation_id: Some(format!("benchmark.operation.{sequence}")),
        operation_display_name: Some("Benchmark Operation".to_string()),
        operation_arguments: Some(json!({ "payload": "x".repeat(payload_bytes) })),
        operation_group: Some("benchmark".to_string()),
        transaction_id: Some(sequence),
        save_generation: None,
        effects: Vec::<EditorEventEffect>::new(),
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: sequence.saturating_sub(1),
        after_revision: sequence,
        result: EditorEventResult::success(json!({ "sequence": sequence })),
    }))
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
