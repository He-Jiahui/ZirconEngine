use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue,
};

use super::{BlackboardLayout, BlackboardSlot, BlackboardStore};

const BENCHMARK_KEY_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;
const BENCHMARK_WRITE_COUNT: usize = 64;

#[test]
fn entry_positions_follow_full_refresh_clear_and_sorted_reinsertion() {
    let (layout, entries) = benchmark_fixture(3);
    let mut store = BlackboardStore::new(layout);
    store.synchronize(&entries).expect("populate store");
    store.drain_changed_slots();

    store
        .synchronize(std::slice::from_ref(&entries[2]))
        .expect("retain final entry");
    store.drain_changed_slots();
    store
        .write("key_0000", AiBlackboardValue::Integer(10))
        .expect("insert before retained entry");
    store
        .write("key_0002", AiBlackboardValue::Integer(22))
        .expect("update shifted retained entry");

    assert_eq!(
        store.entries(),
        [
            AiBlackboardEntry::new("key_0000", AiBlackboardValue::Integer(10)),
            AiBlackboardEntry::new("key_0002", AiBlackboardValue::Integer(22)),
        ]
    );
    for (position, entry) in store.entries_cache.iter().enumerate() {
        let slot = store.layout.resolve(&entry.key).expect("cached key slot");
        assert_eq!(
            store.entry_positions[slot.generation_index() as usize],
            Some(position as u32)
        );
    }
}

#[test]
fn single_entry_refresh_reads_the_dense_position_index_before_binary_search() {
    let source = include_str!("../store.rs");
    let fields = source
        .split("pub struct BlackboardStore {")
        .nth(1)
        .and_then(|body| body.split("impl BlackboardStore").next())
        .expect("BlackboardStore fields");
    let refresh_entry = source
        .split("fn refresh_entry(")
        .nth(1)
        .and_then(|body| body.split("fn refresh_entries").next())
        .expect("refresh_entry body");
    let refresh_entries = source
        .split("fn refresh_entries(")
        .nth(1)
        .and_then(|body| body.split("fn record_changes").next())
        .expect("refresh_entries body");

    assert!(fields.contains("entry_positions: Box<[Option<u32>]>"));
    assert!(refresh_entry.contains("self.entry_positions[generation_index]"));
    assert!(refresh_entries.contains("self.entry_positions.fill(None)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn indexed_blackboard_entry_position_release_benchmark_evidence() {
    let (layout, entries) = benchmark_fixture(BENCHMARK_KEY_COUNT);
    let key = entries[BENCHMARK_KEY_COUNT / 2].key.clone();
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
    let mut legacy_value = BENCHMARK_KEY_COUNT as i64;
    let mut optimized_value = BENCHMARK_KEY_COUNT as i64;

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || binary_search_writes(&mut legacy_store, &key, &mut legacy_value),
        || indexed_writes(&mut optimized_store, &key, &mut optimized_value),
    );
    assert_eq!(legacy_store.entries(), optimized_store.entries());
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_key_comparisons = BENCHMARK_WRITE_COUNT * BENCHMARK_KEY_COUNT.ilog2() as usize;

    println!(
        "PERF_RESULT plugins15_indexed_blackboard_entry_position keys={BENCHMARK_KEY_COUNT} writes_per_sample={BENCHMARK_WRITE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_binary_searches_per_sample={BENCHMARK_WRITE_COUNT} legacy_estimated_key_comparisons_per_sample={legacy_key_comparisons} optimized_position_reads_per_sample={BENCHMARK_WRITE_COUNT} optimized_key_comparisons_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
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

fn binary_search_writes(store: &mut BlackboardStore, key: &str, next_value: &mut i64) -> usize {
    for _ in 0..BENCHMARK_WRITE_COUNT {
        *next_value = next_value.wrapping_add(1);
        let outcome = store
            .write_untracked(
                black_box(key),
                AiBlackboardValue::Integer(black_box(*next_value)),
            )
            .expect("binary search write");
        if outcome.changed {
            binary_search_refresh_entry(store, outcome.slot);
            store.record_changes(std::slice::from_ref(&outcome.slot));
        }
    }
    black_box(store.entries_ref());
    black_box(store.drain_changed_slots().len())
}

fn indexed_writes(store: &mut BlackboardStore, key: &str, next_value: &mut i64) -> usize {
    for _ in 0..BENCHMARK_WRITE_COUNT {
        *next_value = next_value.wrapping_add(1);
        black_box(
            store
                .write(
                    black_box(key),
                    AiBlackboardValue::Integer(black_box(*next_value)),
                )
                .expect("indexed write"),
        );
    }
    black_box(store.entries_ref());
    black_box(store.drain_changed_slots().len())
}

fn binary_search_refresh_entry(store: &mut BlackboardStore, slot: BlackboardSlot) {
    let value = store.read(slot);
    let key = store
        .layout
        .key_for_slot(slot)
        .expect("compiled slot belongs to layout");
    match (
        store
            .entries_cache
            .binary_search_by(|entry| entry.key.as_str().cmp(key)),
        value,
    ) {
        (Ok(index), Some(value)) => store.entries_cache[index].value = value,
        _ => panic!("benchmark starts from a complete populated cache"),
    }
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
