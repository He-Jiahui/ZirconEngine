use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 40_000;
const FALLBACK_RUN_COUNT: usize = 256;

#[test]
fn optimization_batch_20260829al_runtime312_native_indices_cover_the_pending_batch() {
    let indices = optimized_native_fallback_indices(FALLBACK_RUN_COUNT);
    assert_eq!(indices.len(), FALLBACK_RUN_COUNT);
    assert_eq!(indices.capacity(), FALLBACK_RUN_COUNT);
    assert_eq!(indices[0], 0);
    assert_eq!(indices[FALLBACK_RUN_COUNT - 1], FALLBACK_RUN_COUNT - 1);
}

#[test]
fn optimization_batch_20260829al_runtime312_fallback_path_preallocates_native_indices() {
    let source = include_str!("../sdf_fallback.rs");
    let builder = source
        .split("fn apply_sdf_atlas_fallbacks_internal")
        .nth(1)
        .expect("SDF fallback builder")
        .split("impl ScreenSpaceUiTextSdfFallbackReport")
        .next()
        .expect("SDF fallback builder body");

    assert!(
        builder
            .contains("native_fallback_run_indices = Vec::with_capacity(pending_sdf_texts.len())")
    );
    assert!(!builder.contains("native_fallback_run_indices = Vec::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829al_runtime312_preallocated_sdf_native_indices_bench() {
    assert_eq!(
        optimized_native_fallback_indices(FALLBACK_RUN_COUNT),
        legacy_native_fallback_indices(FALLBACK_RUN_COUNT)
    );

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
        "RUNTIME312_PREALLOCATED_SDF_NATIVE_FALLBACK_INDICES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} fallback_runs_per_build={FALLBACK_RUN_COUNT} \
legacy_vector_allocations_per_build=7 optimized_vector_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_native_fallback_indices(run_count: usize) -> Vec<usize> {
    build_native_fallback_indices(Vec::new(), run_count)
}

fn optimized_native_fallback_indices(run_count: usize) -> Vec<usize> {
    build_native_fallback_indices(Vec::with_capacity(run_count), run_count)
}

fn build_native_fallback_indices(mut indices: Vec<usize>, run_count: usize) -> Vec<usize> {
    for index in 0..run_count {
        indices.push(index);
    }
    indices
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let indices = if optimized {
            optimized_native_fallback_indices(black_box(FALLBACK_RUN_COUNT))
        } else {
            legacy_native_fallback_indices(black_box(FALLBACK_RUN_COUNT))
        };
        checksum = checksum.wrapping_add(black_box(indices).len());
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
