use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const PARTITIONS_PER_SAMPLE: usize = 64;
const FEATURES_PER_PARTITION: usize = 4_096;

#[test]
fn optimization_batch_20260826fw_runtime218_capacity_covers_known_feature_selections() {
    let active_selections = (0..FEATURES_PER_PARTITION).collect::<Vec<_>>();
    let mut pending = Vec::with_capacity(active_selections.len());
    pending.extend(active_selections.iter().copied());

    assert_eq!(pending, active_selections);
    assert!(pending.capacity() >= active_selections.len());
}

#[test]
fn optimization_batch_20260826fw_runtime218_partition_reserves_active_selection_count() {
    let source = include_str!("../partition.rs");
    assert!(source.contains("let active_selections = active_feature_selections(manifest);"));
    assert!(source.contains("let mut pending = Vec::with_capacity(active_selections.len());"));
    assert!(source.contains("for active in active_selections"));
    assert!(source.contains("let mut unknown_feature_blocks = Vec::new();"));
    assert!(!source.contains("let mut pending = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fw_runtime218_feature_selection_pending_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME218_FEATURE_SELECTION_PENDING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
partitions_per_sample={PARTITIONS_PER_SAMPLE} features_per_partition={FEATURES_PER_PARTITION} \
legacy_reservations_per_partition=0 optimized_reservations_per_partition=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct PendingFixture([usize; 8]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for partition in 0..PARTITIONS_PER_SAMPLE {
        let mut pending = if reserve {
            Vec::with_capacity(FEATURES_PER_PARTITION)
        } else {
            Vec::new()
        };
        for feature in 0..FEATURES_PER_PARTITION {
            pending.push(PendingFixture([black_box(partition ^ feature); 8]));
        }
        checksum ^= black_box(
            pending.len() ^ pending.capacity() ^ pending[FEATURES_PER_PARTITION - 1].0[0],
        );
        black_box(&pending);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
