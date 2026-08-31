use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const PROJECTIONS_PER_SAMPLE: usize = 64;
const ROWS_PER_PROJECTION: usize = 4_096;

#[test]
fn optimization_batch_20260826gc_runtime224_manifest_index_capacity_covers_input_rows() {
    let mut selection_indices = HashMap::with_capacity(ROWS_PER_PROJECTION);
    let mut feature_ids = HashSet::with_capacity(ROWS_PER_PROJECTION);
    for row in 0..ROWS_PER_PROJECTION {
        selection_indices.insert(row, row * 2);
        feature_ids.insert(row);
    }

    assert_eq!(selection_indices.len(), ROWS_PER_PROJECTION);
    assert_eq!(feature_ids.len(), ROWS_PER_PROJECTION);
    assert!(selection_indices.capacity() >= ROWS_PER_PROJECTION);
    assert!(feature_ids.capacity() >= ROWS_PER_PROJECTION);
}

#[test]
fn optimization_batch_20260826gc_runtime224_manifest_projection_reserves_known_row_counts() {
    let source = include_str!("../projection.rs");

    assert_eq!(
        source
            .matches("with_capacity(manifest.selections.len())")
            .count(),
        4
    );
    assert_eq!(
        source
            .matches("with_capacity(selection.features.len())")
            .count(),
        4
    );
    assert!(source.contains("let mut feature_locations = HashMap"));
    assert!(source.contains("let mut short_feature_ids = HashSet::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gc_runtime224_manifest_validation_index_capacity_bench() {
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
        "RUNTIME224_MANIFEST_VALIDATION_INDEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} rows_per_projection={ROWS_PER_PROJECTION} \
legacy_preallocated_indices_per_projection=0 optimized_preallocated_indices_per_projection=3 \
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
    for projection in 0..PROJECTIONS_PER_SAMPLE {
        let mut selection_indices = if reserve {
            HashMap::with_capacity(ROWS_PER_PROJECTION)
        } else {
            HashMap::new()
        };
        let mut feature_indices = if reserve {
            HashMap::with_capacity(ROWS_PER_PROJECTION)
        } else {
            HashMap::new()
        };
        let mut feature_ids = if reserve {
            HashSet::with_capacity(ROWS_PER_PROJECTION)
        } else {
            HashSet::new()
        };
        for row in 0..ROWS_PER_PROJECTION {
            let key = black_box(projection ^ row);
            selection_indices.insert(key, [key; 4]);
            feature_indices.insert(key, [key; 4]);
            feature_ids.insert(key);
        }
        checksum ^= black_box(
            selection_indices.capacity() ^ feature_indices.capacity() ^ feature_ids.capacity(),
        );
        black_box((&selection_indices, &feature_indices, &feature_ids));
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
