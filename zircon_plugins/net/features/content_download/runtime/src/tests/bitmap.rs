use std::{hint::black_box, time::Instant};

use super::indexed_completion_bitmap;

const BENCHMARK_CHUNK_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn indexed_completion_bitmap_preserves_manifest_order_and_duplicate_semantics() {
    let manifest = ["chunk-a", "chunk-b", "chunk-c", "chunk-d"];
    let completed = vec![
        "chunk-c".to_string(),
        "chunk-a".to_string(),
        "chunk-c".to_string(),
        "unrelated".to_string(),
    ];

    assert_eq!(
        indexed_completion_bitmap(manifest, &completed),
        vec![true, false, true, false]
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn indexed_completion_bitmap_release_benchmark_evidence() {
    let manifest = (0..BENCHMARK_CHUNK_COUNT)
        .map(|index| format!("chunk-{index:05}"))
        .collect::<Vec<_>>();
    let completed = manifest.iter().step_by(2).cloned().collect::<Vec<_>>();
    assert_eq!(
        legacy_completion_bitmap(&manifest, &completed),
        indexed_completion_bitmap(manifest.iter().map(String::as_str), &completed)
    );

    let legacy_string_comparisons = completed.len() * (completed.len() + 1) / 2
        + (manifest.len() - completed.len()) * completed.len();
    let optimized_hash_lookups = manifest.len();
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy(&manifest, &completed));
            optimized_samples.push(measure_optimized(&manifest, &completed));
        } else {
            optimized_samples.push(measure_optimized(&manifest, &completed));
            legacy_samples.push(measure_legacy(&manifest, &completed));
        }
    }

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT task=plugins10_indexed_resume_bitmap chunks={BENCHMARK_CHUNK_COUNT} completed_chunks={} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_string_comparisons_per_sample={legacy_string_comparisons} optimized_hash_lookups_per_sample={optimized_hash_lookups} threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
        completed.len(),
        raw_samples(&legacy_samples),
        raw_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(50),
        "indexed resume bitmap P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 50%"
    );
}

fn legacy_completion_bitmap(manifest: &[String], completed: &[String]) -> Vec<bool> {
    manifest
        .iter()
        .map(|chunk_id| completed.iter().any(|id| id == chunk_id))
        .collect()
}

fn measure_legacy(manifest: &[String], completed: &[String]) -> u128 {
    let start = Instant::now();
    let result = legacy_completion_bitmap(black_box(manifest), black_box(completed));
    let elapsed = start.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn measure_optimized(manifest: &[String], completed: &[String]) -> u128 {
    let start = Instant::now();
    let result = indexed_completion_bitmap(
        black_box(manifest).iter().map(String::as_str),
        black_box(completed),
    );
    let elapsed = start.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u128]) -> String {
    format!(
        "[{}]",
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}
