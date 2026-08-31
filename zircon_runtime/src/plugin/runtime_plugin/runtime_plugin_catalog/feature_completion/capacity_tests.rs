use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const MAPS_PER_SAMPLE: usize = 64;
const FEATURES_PER_MAP: usize = 4_096;

#[test]
fn optimization_batch_20260826fy_runtime220_feature_index_capacity_covers_existing_features() {
    let mut feature_indices = HashMap::with_capacity(FEATURES_PER_MAP);
    for feature in 0..FEATURES_PER_MAP {
        feature_indices.entry(feature).or_insert(feature * 2);
    }

    assert_eq!(feature_indices.len(), FEATURES_PER_MAP);
    assert!(feature_indices.capacity() >= FEATURES_PER_MAP);
    assert_eq!(feature_indices[&0], 0);
    assert_eq!(
        feature_indices[&(FEATURES_PER_MAP - 1)],
        (FEATURES_PER_MAP - 1) * 2
    );
}

#[test]
fn optimization_batch_20260826fy_runtime220_feature_completion_reserves_selection_features() {
    let source = include_str!("../feature_completion.rs");

    assert!(source.contains("HashMap::with_capacity(selection.features.len())"));
    assert_eq!(
        source
            .matches("HashMap::with_capacity(selection.features.len())")
            .count(),
        1
    );
    assert!(!source.contains("let mut feature_indices = HashMap::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fy_runtime220_feature_index_capacity_bench() {
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
        "RUNTIME220_FEATURE_INDEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
maps_per_sample={MAPS_PER_SAMPLE} features_per_map={FEATURES_PER_MAP} \
legacy_preallocated_maps_per_selection=0 optimized_preallocated_maps_per_selection=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for map in 0..MAPS_PER_SAMPLE {
        let mut feature_indices = if reserve {
            HashMap::with_capacity(FEATURES_PER_MAP)
        } else {
            HashMap::new()
        };
        for feature in 0..FEATURES_PER_MAP {
            let feature = black_box(map ^ feature);
            feature_indices.entry(feature).or_insert(feature * 2);
        }
        checksum ^= black_box(feature_indices.len() ^ feature_indices.capacity());
        black_box(&feature_indices);
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
