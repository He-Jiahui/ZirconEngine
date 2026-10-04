use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{normalize_text_filter, text_matches_filter};

const PERFORMANCE_MARKER: &str = "RUNTIME139_MATERIAL_QUERY_BORROWED_FILTER_BENCH_V1";

#[test]
fn optimization_batch_20260826cv_runtime139_borrowed_filter_preserves_matching_semantics() {
    let query = "  WeAtHeReD_MaTeRiAl  ";
    let normalized = normalize_text_filter(query).expect("trimmed query should remain");

    assert_eq!(normalized, "WeAtHeReD_MaTeRiAl");
    assert!(text_matches_filter(
        "Environment/Weathered_Material_07",
        normalized
    ));
    assert!(!text_matches_filter(
        "Environment/PolishedMetal_07",
        normalized
    ));
}

#[test]
fn optimization_batch_20260826cv_runtime139_borrowed_filter_reuses_trimmed_storage() {
    let query = String::from("  MAT-Preview-Query  ");
    let normalized = normalize_text_filter(&query).expect("trimmed query should remain");
    let expected_offset = query
        .find('M')
        .expect("query should contain its first token");

    assert_eq!(normalized.as_ptr(), query[expected_offset..].as_ptr());
    assert_eq!(normalize_text_filter(" \t\r\n "), None);
}

#[test]
#[ignore = "release-only material query filter performance gate"]
fn optimization_batch_20260826cv_runtime139_borrowed_filter_performance_evidence() {
    const QUERY_COUNT: usize = 16_384;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME139_MATERIAL_QUERY_BORROWED_FILTER_BENCH_V1"
    );
    let queries = (0..QUERY_COUNT)
        .map(|index| {
            format!(
                "   Environment/Weathered_Material_Instance_{index:08}_RuntimePreview_Profile   "
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(legacy_normalize_batch(&queries));
        black_box(borrowed_normalize_batch(&queries));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| black_box(legacy_normalize_batch(&queries))));
            optimized_samples.push(measure(|| black_box(borrowed_normalize_batch(&queries))));
        } else {
            optimized_samples.push(measure(|| black_box(borrowed_normalize_batch(&queries))));
            legacy_samples.push(measure(|| black_box(legacy_normalize_batch(&queries))));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} queries={QUERY_COUNT} samples={SAMPLE_COUNT} legacy_filter_allocations={QUERY_COUNT} optimized_filter_allocations=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed filter P95 {optimized_p95_ns}ns must be at most 70% of allocated lowercase filter P95 {legacy_p95_ns}ns"
    );
}

fn legacy_normalize_batch(queries: &[String]) -> usize {
    queries
        .iter()
        .map(|query| {
            let normalized = query.trim().to_ascii_lowercase();
            black_box(normalized.len())
        })
        .sum()
}

fn borrowed_normalize_batch(queries: &[String]) -> usize {
    queries
        .iter()
        .map(|query| {
            black_box(
                normalize_text_filter(black_box(query))
                    .expect("benchmark query should remain")
                    .len(),
            )
        })
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
