use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue,
};

use super::{BlackboardLayout, BlackboardRuntimeError, BlackboardSlot, BlackboardStore};

const BENCHMARK_KEY_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn synchronize_scratch_survives_validation_failure_and_epoch_wrap() {
    let (layout, entries) = benchmark_fixture(3);
    let mut store = BlackboardStore::new(layout);
    store.synchronize(&entries).expect("initial synchronize");
    store.drain_changed_slots();
    let entries_before = store.entries();
    let slot_capacity = store.synchronize_slots.capacity();
    let cache_capacity = store.entries_cache.capacity();

    let error = store
        .synchronize(&[
            AiBlackboardEntry::new("key_0001", AiBlackboardValue::Integer(11)),
            AiBlackboardEntry::new("key_0001", AiBlackboardValue::Integer(12)),
        ])
        .expect_err("duplicate remains fail closed");
    assert_eq!(
        error,
        BlackboardRuntimeError::DuplicateKey {
            key: "key_0001".to_string(),
        }
    );
    assert_eq!(store.entries(), entries_before);

    store.synchronize_epoch = u32::MAX;
    store.synchronize_marks.fill(u32::MAX);
    let changed = store
        .synchronize(&entries)
        .expect("wrapped epoch synchronize");
    assert!(changed.is_empty());
    assert_eq!(store.synchronize_epoch, 1);
    assert_eq!(store.synchronize_slots.capacity(), slot_capacity);
    assert_eq!(store.entries_cache.capacity(), cache_capacity);
}

#[test]
fn synchronize_reuses_epoch_marks_slot_scratch_and_entry_cache_storage() {
    let source = include_str!("../store.rs");
    let fields = source
        .split("pub struct BlackboardStore {")
        .nth(1)
        .and_then(|body| body.split("impl BlackboardStore").next())
        .expect("BlackboardStore fields");
    let synchronize = source
        .split("pub fn synchronize(")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn drain_changed_slots").next())
        .expect("synchronize body");
    let refresh_entries = source
        .split("fn refresh_entries(")
        .nth(1)
        .and_then(|body| body.split("fn record_changes").next())
        .expect("refresh_entries body");

    assert!(fields.contains("synchronize_epoch: u32"));
    assert!(fields.contains("synchronize_marks: Box<[u32]>"));
    assert!(fields.contains("synchronize_slots: Vec<BlackboardSlot>"));
    assert!(synchronize.contains("self.next_synchronize_epoch()"));
    assert!(synchronize.contains("self.synchronize_slots.clear()"));
    assert!(!synchronize.contains("vec![false; self.layout.key_count()]"));
    assert!(!synchronize.contains("Vec::with_capacity(entries.len())"));
    assert!(refresh_entries.contains("std::mem::take(&mut self.entries_cache)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn reusable_blackboard_synchronize_scratch_release_benchmark_evidence() {
    let (layout, entries) = benchmark_fixture(BENCHMARK_KEY_COUNT);
    let mut legacy_store = BlackboardStore::new(layout.clone());
    legacy_store
        .synchronize(&entries)
        .expect("populate legacy store");
    legacy_store.drain_changed_slots();
    let mut optimized_store = BlackboardStore::new(layout);
    optimized_store
        .synchronize(&entries)
        .expect("populate optimized store");
    optimized_store.drain_changed_slots();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || allocating_synchronize(&mut legacy_store, black_box(&entries)),
        || reusable_synchronize(&mut optimized_store, black_box(&entries)),
    );
    assert_eq!(legacy_store.entries(), optimized_store.entries());
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_reusable_blackboard_synchronize_scratch entries={BENCHMARK_KEY_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_work_buffer_allocations_per_sample=2 optimized_work_buffer_allocations_per_sample=0 legacy_mark_zero_initializations_per_sample={BENCHMARK_KEY_COUNT} optimized_mark_zero_initializations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 20 <= legacy_p95 * 19,
        "optimized P95 {optimized_p95}ns must be no more than 95% of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_fixture(key_count: usize) -> (Arc<BlackboardLayout>, Vec<AiBlackboardEntry>) {
    let mut descriptor = AiBlackboardSchemaDescriptor::new("benchmark", "Benchmark");
    let mut entries = Vec::with_capacity(key_count);
    for index in 0..key_count {
        let key = format!("key_{index:04}");
        descriptor = descriptor.with_key(key.clone(), "integer", false);
        entries.push(AiBlackboardEntry::new(
            key,
            AiBlackboardValue::Integer(index as i64),
        ));
    }
    let layout = Arc::new(BlackboardLayout::from_schema(&descriptor).expect("valid layout"));
    (layout, entries)
}

fn allocating_synchronize(store: &mut BlackboardStore, entries: &[AiBlackboardEntry]) -> usize {
    let mut present = vec![false; store.layout.key_count()];
    let mut validated_slots = Vec::with_capacity(entries.len());
    for entry in entries {
        let slot = store.resolve_slot(&entry.key).expect("known key");
        let generation_index = slot.generation_index() as usize;
        assert!(!present[generation_index]);
        store
            .validate_slot_value(&entry.key, slot, &entry.value)
            .expect("valid value");
        present[generation_index] = true;
        validated_slots.push(slot);
    }
    apply_synchronize(store, entries, &validated_slots, |slot| {
        present[slot.generation_index() as usize]
    })
}

fn reusable_synchronize(store: &mut BlackboardStore, entries: &[AiBlackboardEntry]) -> usize {
    let changed = store.synchronize(entries).expect("reusable synchronize");
    black_box(store.entries_ref());
    black_box(changed.len())
}

fn apply_synchronize(
    store: &mut BlackboardStore,
    entries: &[AiBlackboardEntry],
    validated_slots: &[BlackboardSlot],
    is_present: impl Fn(BlackboardSlot) -> bool,
) -> usize {
    let mut changed = Vec::new();
    for (entry, slot) in entries.iter().zip(validated_slots.iter().copied()) {
        let outcome = store.write_validated(slot, entry.value.clone());
        if outcome.changed {
            changed.push(outcome.slot);
        }
    }
    let layout = Arc::clone(&store.layout);
    for (_, slot) in layout.slots() {
        if !is_present(slot) && store.clear(slot) {
            let generation = &mut store.generations[slot.generation_index() as usize];
            *generation = generation.wrapping_add(1);
            changed.push(slot);
        }
    }
    if !changed.is_empty() {
        store.refresh_entries();
        store.record_changes(&changed);
    }
    black_box(store.entries_ref());
    black_box(changed.len())
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
