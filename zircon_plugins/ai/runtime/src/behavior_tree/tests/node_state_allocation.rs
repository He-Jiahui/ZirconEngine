use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn steady_node_state_lookup_borrows_tree_id_before_allocating_a_key() {
    let source = include_str!("../executor.rs");
    let node_mut = source
        .split("fn node_mut(")
        .nth(1)
        .and_then(|body| body.split("fn bind_observers(").next())
        .expect("node_mut body");

    assert!(node_mut.contains("self.trees.get_mut(tree.id())"));
    assert!(!node_mut.contains(".entry(tree.id().to_string())"));
    assert_eq!(node_mut.matches("tree.id().to_string()").count(), 1);
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_node_state_lookup_release_benchmark_evidence() {
    let tree_id = format!("tree-{}", "x".repeat(96));
    let mut legacy = state_map(tree_id.as_str());
    let mut optimized = legacy.clone();
    assert_eq!(
        legacy_lookup_checksum(&mut legacy, tree_id.as_str()),
        borrowed_lookup_checksum(&mut optimized, tree_id.as_str())
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_lookup_checksum(black_box(&mut legacy), black_box(tree_id.as_str())),
        || borrowed_lookup_checksum(black_box(&mut optimized), black_box(tree_id.as_str())),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_borrowed_node_state_lookup lookups={BENCHMARK_LOOKUP_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_string_allocations_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 40% of legacy P95 {legacy_p95}ns"
    );
}

fn state_map(tree_id: &str) -> BTreeMap<String, Vec<u32>> {
    BTreeMap::from([(tree_id.to_string(), vec![7, 11, 13, 17])])
}

fn legacy_lookup_checksum(states: &mut BTreeMap<String, Vec<u32>>, tree_id: &str) -> u64 {
    let mut checksum = 0_u64;
    for lookup in 0..BENCHMARK_LOOKUP_COUNT {
        let values = states
            .entry(tree_id.to_string())
            .or_insert_with(|| vec![7, 11, 13, 17]);
        checksum += u64::from(values[lookup % values.len()]);
    }
    checksum
}

fn borrowed_lookup_checksum(states: &mut BTreeMap<String, Vec<u32>>, tree_id: &str) -> u64 {
    let mut checksum = 0_u64;
    for lookup in 0..BENCHMARK_LOOKUP_COUNT {
        assert!(states.contains_key(tree_id));
        let values = states.get_mut(tree_id).expect("preloaded tree state");
        checksum += u64::from(values[lookup % values.len()]);
    }
    checksum
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
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

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
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
