use std::hint::black_box;
use std::time::Instant;

use super::{FontCoverage, normalize_codepoint_values};

const MARKER: &str = "RUNTIME245_FONT_COVERAGE_HASH_DEDUP_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 512;

#[test]
fn optimization_batch_20260826gy_runtime245_font_coverage_stays_unique_and_compact() {
    assert_eq!(
        FontCoverage::from_codepoint_values(vec![40, 33, 32, 33, 34, 35, 40]),
        FontCoverage::Known(vec![(32, 35), (40, 40)])
    );
}

#[test]
fn optimization_batch_20260826gy_runtime245_font_coverage_dedups_large_inputs_before_sorting() {
    let source = include_str!("../coverage.rs");
    let implementation = source
        .split("fn normalize_codepoint_values")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("font coverage normalization implementation");
    assert!(implementation.contains("HASH_DEDUP_CODEPOINT_THRESHOLD"));
    assert!(implementation.contains("HashSet::with_capacity"));
    assert!(implementation.contains("unique.insert(codepoint)"));
    assert!(implementation.contains("unique.into_iter().collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gy_runtime245_font_coverage_hash_dedup_bench() {
    let codepoints = (0..4_096)
        .map(|index| 0x20 + (index % 16) as u32)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&codepoints, legacy_font_coverage));
            optimized_samples.push(measure(&codepoints, optimized_font_coverage));
        } else {
            optimized_samples.push(measure(&codepoints, optimized_font_coverage));
            legacy_samples.push(measure(&codepoints, legacy_font_coverage));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "hash deduplication must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_font_coverage(codepoints: &[u32]) -> FontCoverage {
    let mut codepoints = codepoints.to_vec();
    codepoints.sort_unstable();
    codepoints.dedup();
    FontCoverage::from_sorted_unique_codepoints(codepoints)
}

fn optimized_font_coverage(codepoints: &[u32]) -> FontCoverage {
    FontCoverage::from_sorted_unique_codepoints(normalize_codepoint_values(codepoints.to_vec()))
}

fn measure(codepoints: &[u32], implementation: fn(&[u32]) -> FontCoverage) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let coverage = implementation(black_box(codepoints));
        checksum = checksum.wrapping_add(match &coverage {
            FontCoverage::Known(ranges) => ranges.len(),
            FontCoverage::Unknown => 0,
        });
        black_box(&coverage);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
