use std::hint::black_box;
use std::time::Instant;

use super::{normalized_text_filter, normalized_text_filter_ref};

const SAMPLE_PAIRS: usize = 21;
const CHECKS_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826ds_runtime162_material_query_filter_preserves_normalization() {
    let populated = Some("  hero_surface  ".to_string());
    assert_eq!(normalized_text_filter_ref(&populated), Some("hero_surface"));
    assert_eq!(
        normalized_text_filter(&populated),
        Some("hero_surface".to_string())
    );
    assert_eq!(normalized_text_filter_ref(&Some("   ".to_string())), None);
    assert_eq!(normalized_text_filter_ref(&None), None);
}

#[test]
fn optimization_batch_20260826ds_runtime162_material_query_filter_borrows_trimmed_slice() {
    let populated = Some("  production_material  ".to_string());
    let original = populated.as_deref().unwrap();
    let normalized = normalized_text_filter_ref(&populated).unwrap();
    assert_eq!(normalized.as_ptr(), original.as_ptr().wrapping_add(2));

    let source = include_str!("../query_state.rs");
    assert!(source.contains("normalized_text_filter_ref(&self.text_filter).is_some()"));
    assert!(source.contains("normalized_text_filter_ref(text_filter).map(str::to_string)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ds_runtime162_material_query_borrowed_filter_check_bench() {
    let filter = Some("  production_material_surface_variant  ".to_string());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&filter));
            optimized_samples.push(measure_optimized(&filter));
        } else {
            optimized_samples.push(measure_optimized(&filter));
            legacy_samples.push(measure_legacy(&filter));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME162_MATERIAL_QUERY_BORROWED_FILTER_CHECK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} legacy_allocations_per_check=1 \
optimized_allocations_per_check=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed material filter check P95 {optimized_p95_ns}ns must be at most 70% of owned normalization P95 {legacy_p95_ns}ns"
    );
}

fn legacy_normalized_text_filter(text_filter: &Option<String>) -> Option<String> {
    text_filter
        .as_deref()
        .map(str::trim)
        .filter(|text_filter| !text_filter.is_empty())
        .map(str::to_string)
}

fn measure_legacy(filter: &Option<String>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        checksum ^= black_box(legacy_normalized_text_filter(black_box(filter)))
            .unwrap()
            .len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(filter: &Option<String>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        checksum ^= black_box(normalized_text_filter_ref(black_box(filter)))
            .unwrap()
            .len();
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
