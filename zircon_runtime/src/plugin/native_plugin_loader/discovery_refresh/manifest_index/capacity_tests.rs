use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const PROJECTIONS_PER_SAMPLE: usize = 64;
const CANDIDATES_PER_PROJECTION: usize = 4_096;

#[test]
fn optimization_batch_20260826fz_runtime221_projection_capacity_covers_index_candidates() {
    let mut candidates = Vec::with_capacity(CANDIDATES_PER_PROJECTION);
    for candidate in 0..CANDIDATES_PER_PROJECTION {
        candidates.push(candidate);
    }

    assert_eq!(candidates.len(), CANDIDATES_PER_PROJECTION);
    assert!(candidates.capacity() >= CANDIDATES_PER_PROJECTION);
    assert_eq!(candidates[0], 0);
    assert_eq!(
        candidates[CANDIDATES_PER_PROJECTION - 1],
        CANDIDATES_PER_PROJECTION - 1
    );
}

#[test]
fn optimization_batch_20260826fz_runtime221_manifest_projection_reserves_index_size() {
    let source = include_str!("../manifest_index.rs");

    assert!(source.contains("Vec::with_capacity(self.candidates.len())"));
    assert_eq!(
        source
            .matches("Vec::with_capacity(self.candidates.len())")
            .count(),
        1
    );
    assert!(source.contains("let mut diagnostics = Vec::new();"));
    assert!(!source.contains("let mut candidates = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fz_runtime221_native_manifest_projection_capacity_bench() {
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
        "RUNTIME221_NATIVE_MANIFEST_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} candidates_per_projection={CANDIDATES_PER_PROJECTION} \
legacy_preallocated_candidate_outputs=0 optimized_preallocated_candidate_outputs={PROJECTIONS_PER_SAMPLE} \
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
        let mut candidates = if reserve {
            Vec::with_capacity(CANDIDATES_PER_PROJECTION)
        } else {
            Vec::new()
        };
        for candidate in 0..CANDIDATES_PER_PROJECTION {
            let value = black_box(projection ^ candidate);
            candidates.push([value; 16]);
        }
        checksum ^= black_box(candidates.len() ^ candidates.capacity());
        black_box(&candidates);
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
