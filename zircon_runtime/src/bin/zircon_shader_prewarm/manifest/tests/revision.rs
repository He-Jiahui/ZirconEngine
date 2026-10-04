use std::hint::black_box;
use std::time::Instant;

use super::*;

fn legacy_content_revision(include_content_hashes: &[String]) -> u64 {
    if include_content_hashes.is_empty() {
        return ASSET_SCAN_INITIAL_RESOURCE_REVISION;
    }

    let mut hasher = blake3::Hasher::new();
    for hash in include_content_hashes {
        hasher.update(hash.as_bytes());
        hasher.update(&[0]);
    }
    non_zero_revision_from_hash(hasher.finalize())
}

fn legacy_base_revision(base_revision: u64, include_content_hashes: &[String]) -> u64 {
    if include_content_hashes.is_empty() {
        return base_revision;
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(&base_revision.to_le_bytes());
    hasher.update(&[0]);
    for hash in include_content_hashes {
        hasher.update(hash.as_bytes());
        hasher.update(&[0]);
    }
    non_zero_revision_from_hash(hasher.finalize())
}

#[test]
fn optimization_batch_20260830ey_runtime564_preserves_content_revision_bytes() {
    let values = vec![
        String::new(),
        "short".to_owned(),
        "x".repeat(64),
        "y".repeat(65),
        "z".repeat(129),
    ];
    assert_eq!(
        asset_scan_revision_from_content_hashes(&values),
        legacy_content_revision(&values)
    );
}

#[test]
fn optimization_batch_20260830ey_runtime564_preserves_base_revision_bytes() {
    let values = vec!["a".repeat(64), "b".repeat(65), "c".repeat(3)];
    assert_eq!(
        asset_scan_revision_from_base_revision_and_content_hashes(41, &values),
        legacy_base_revision(41, &values)
    );
    assert_eq!(
        asset_scan_revision_from_base_revision_and_content_hashes(41, &[]),
        41
    );
}

#[test]
fn optimization_batch_20260830ey_runtime564_batches_short_values_into_one_update() {
    let source = include_str!("../revision.rs");
    assert!(source.contains("buffered[value.len()] = 0"));
    assert!(source.contains("hasher.update(&buffered[..value.len() + 1])"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ey_runtime564_delimited_hash_p95() {
    const SAMPLE_PAIRS: usize = 13;
    const HASHES_PER_SAMPLE: usize = 256;
    const ITERATIONS: u64 = 4_096;
    let values = (0..HASHES_PER_SAMPLE)
        .map(|index| format!("{index:064x}"))
        .collect::<Vec<_>>();
    let base_revision = 41;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure_content_revision(&values, false, ITERATIONS));
            optimized.push(measure_content_revision(&values, true, ITERATIONS));
        } else {
            optimized.push(measure_content_revision(&values, true, ITERATIONS));
            legacy.push(measure_content_revision(&values, false, ITERATIONS));
        }
    }

    assert_eq!(
        asset_scan_revision_from_content_hashes(&values),
        legacy_content_revision(&values)
    );
    assert_eq!(
        asset_scan_revision_from_base_revision_and_content_hashes(base_revision, &values),
        legacy_base_revision(base_revision, &values)
    );

    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME564_SHADER_PREWARM_DELIMITED_HASH_BENCH_V1 \
sample_pairs={SAMPLE_PAIRS} hashes_per_sample={HASHES_PER_SAMPLE} iterations={ITERATIONS} \
legacy_updates_per_iteration={} optimized_updates_per_iteration={} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        HASHES_PER_SAMPLE * 2,
        HASHES_PER_SAMPLE,
        csv(&legacy),
        csv(&optimized)
    );
}

fn measure_content_revision(values: &[String], optimized: bool, iterations: u64) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        let revision = if optimized {
            asset_scan_revision_from_content_hashes(black_box(values))
        } else {
            legacy_content_revision(black_box(values))
        };
        checksum ^= black_box(revision);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
