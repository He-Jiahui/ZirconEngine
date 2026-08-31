use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;

use super::*;
use crate::core::editor_event::{
    EditorEventEffect, EditorEventId, EditorEventResult, EditorEventSequence, EditorEventSource,
    EditorEventUndoPolicy,
};

const SAMPLE_PAIRS: usize = 21;
const ENTRY_COUNT: usize = 65_536;

fn payload(sequence: u64, event: EditorEvent) -> Arc<SharedEditorEventRecord> {
    Arc::new(SharedEditorEventRecord::new(EditorEventRecord {
        event_id: EditorEventId::new(sequence),
        sequence: EditorEventSequence::new(sequence),
        source: EditorEventSource::Headless,
        event,
        binding_path: None,
        operation_id: None,
        operation_display_name: None,
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: Vec::<EditorEventEffect>::new(),
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: sequence.saturating_sub(1),
        after_revision: sequence,
        result: EditorEventResult::success(json!(null)),
    }))
}

fn budgets(max_records: usize) -> EditorEventRetentionBudgets {
    let make = || {
        EditorEventRetentionBudget::new(max_records, usize::MAX, Duration::from_secs(60))
            .expect("valid retention budget")
    };
    EditorEventRetentionBudgets::new(make(), make(), make())
}

#[test]
fn optimization_batch_20260826cj_editor_hash_delivery_queue_preserves_pages_and_coalescing() {
    let mut store = EditorEventRetentionStore::new(budgets(128));
    for sequence in 1..=32 {
        store.push(payload(
            sequence,
            EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
        ));
    }
    for sequence in 33..=1_056 {
        store.push(payload(
            sequence,
            EditorEvent::Viewport(EditorViewportEvent::PointerMoved {
                x: sequence as f32,
                y: 0.0,
            }),
        ));
    }

    let page = store.records_page_after_delivery_cursor(8, 5);
    assert_eq!(
        page.records
            .iter()
            .map(|record| record.payload.record().sequence.0)
            .collect::<Vec<_>>(),
        vec![9, 10, 11, 12, 13]
    );
    assert!(page.has_more);
    assert_eq!(store.latest_state.entries.len(), 1);
    assert!(store.latest_state.delivery_order.len() <= 65);
    assert_eq!(store.acknowledge_through_delivery_cursor(32), 32);
    assert_eq!(
        store
            .records()
            .iter()
            .map(|record| record.record().sequence.0)
            .collect::<Vec<_>>(),
        vec![1_056]
    );

    let mut saturated = EditorEventRetentionStore::new(budgets(8));
    saturated.next_delivery_cursor = u64::MAX - 1;
    for sequence in 1..=2 {
        saturated.push(payload(
            sequence,
            EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
        ));
    }
    assert_eq!(saturated.durable_replay.delivery_order.len(), 1);
    assert_eq!(
        saturated.records()[0].record().sequence.0,
        2,
        "a saturated delivery cursor keeps latest replacement semantics"
    );
}

#[test]
fn optimization_batch_20260826cj_editor_retention_payload_index_is_hash_based_and_order_explicit() {
    let source = include_str!("../retention.rs");
    let queue = source
        .split("struct RetentionQueue")
        .nth(1)
        .and_then(|body| body.split("impl RetentionQueue").next())
        .expect("retention queue fields");

    assert!(queue.contains("entries: HashMap<u64, RetainedEditorEvent>"));
    assert!(queue.contains("delivery_order: VecDeque<u64>"));
    assert!(!queue.contains("entries: BTreeMap"));
}

fn insertion_keys() -> Vec<u64> {
    (0..ENTRY_COUNT)
        .map(|index| ((index * 65_537) % ENTRY_COUNT) as u64)
        .collect()
}

fn measure_legacy(keys: &[u64]) -> u128 {
    let started = Instant::now();
    let mut entries = BTreeMap::new();
    for (index, key) in keys.iter().copied().enumerate() {
        entries.insert(key, index);
    }
    black_box(entries.len());
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(keys: &[u64]) -> u128 {
    let started = Instant::now();
    let mut entries = HashMap::with_capacity(keys.len());
    let mut delivery_order = VecDeque::with_capacity(keys.len());
    for (index, key) in keys.iter().copied().enumerate() {
        entries.insert(key, index);
        delivery_order.push_back(key);
    }
    black_box((entries.len(), delivery_order.len()));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
#[ignore = "release-only event retention payload index benchmark"]
fn optimization_batch_20260826cj_editor_retention_hash_delivery_queue_release_benchmark() {
    let keys = insertion_keys();
    for _ in 0..4 {
        black_box(measure_legacy(&keys));
        black_box(measure_optimized(&keys));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&keys));
            optimized_samples.push(measure_optimized(&keys));
        } else {
            optimized_samples.push(measure_optimized(&keys));
            legacy_samples.push(measure_legacy(&keys));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR49_EVENT_RETENTION_HASH_DELIVERY_QUEUE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
entry_count={ENTRY_COUNT} pair_order=alternating_legacy_even legacy_first_pairs=11 \
optimized_first_pairs=10 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "hash payload index must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
