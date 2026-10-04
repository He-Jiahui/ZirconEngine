use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const PATH_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_PATH_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn asset_paths() -> Vec<String> {
    (0..PATH_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "generated/assets/with/a/long/shared/prefix/artifact_{:05}.bin",
                (index * 4_099) % UNIQUE_PATH_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(paths: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    paths
        .iter()
        .filter(|path| unique.insert(path.as_str()))
        .count()
}

fn hash_unique_count(paths: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(paths.len());
    paths
        .iter()
        .filter(|path| unique.insert(path.as_str()))
        .count()
}

fn legacy_chunk_coverage(hashes: &[[u8; 32]]) -> bool {
    let mut seen_chunk_hashes = HashSet::with_capacity(hashes.len());
    seen_chunk_hashes.extend(hashes.iter().copied());
    let asset_hashes = hashes.iter().copied().collect::<HashSet<_>>();
    let chunk_hashes = hashes.iter().copied().collect::<HashSet<_>>();
    black_box(seen_chunk_hashes.len() == hashes.len() && asset_hashes == chunk_hashes)
}

fn reused_chunk_coverage(hashes: &[[u8; 32]]) -> bool {
    let mut seen_chunk_hashes = HashSet::with_capacity(hashes.len());
    seen_chunk_hashes.extend(hashes.iter().copied());
    let asset_hashes = hashes.iter().copied().collect::<HashSet<_>>();
    black_box(asset_hashes == seen_chunk_hashes)
}

#[test]
fn optimization_batch_20260826ab_runtime04_hash_manifest_validation_preserves_first_duplicate_error(
) {
    let assets = vec![
        ZrPackAssetEntry::new("assets/a.bin", [1; 32], 1),
        ZrPackAssetEntry::new("assets/b.bin", [2; 32], 1),
        ZrPackAssetEntry::new("assets/a.bin", [3; 32], 1),
    ];

    assert_eq!(
        validate_zrpack_asset_entries(&assets),
        Err(ZrPackError::DuplicateAssetPath("assets/a.bin".to_string()))
    );
}

#[test]
fn optimization_batch_20260826ab_runtime04_pack_manifest_uses_hash_membership_and_sorted_windows() {
    let source = include_str!("../manifest.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert_eq!(production.matches("HashSet::with_capacity(").count(), 4);
    assert!(!production.contains("collect::<HashSet<_>>()"));
    assert!(production.matches(".windows(2)").count() >= 2);
    assert!(!production.contains("BTreeSet"));
}

#[test]
fn optimization_batch_ie_runtime615_manifest_reuses_validated_chunk_hash_index() {
    let source = include_str!("../manifest.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("HashSet::with_capacity(manifest.pack.chunks.len())"));
    assert!(production.contains("seen_chunk_hashes.insert(chunk.hash)"));
    assert!(production.contains("asset_hashes != seen_chunk_hashes"));
    assert!(!production.contains("chunk_sizes.keys().copied().collect::<HashSet<_>>()"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ab_runtime04_pack_manifest_hash_validation_performance_evidence() {
    let paths = asset_paths();
    assert_eq!(ordered_unique_count(&paths), hash_unique_count(&paths));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&paths)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&paths)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&paths)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&paths)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME04_PACK_MANIFEST_HASH_VALIDATION_BENCH_V1 \
             admissions={PATH_ADMISSION_COUNT} unique_paths={UNIQUE_PATH_COUNT} \
             borrowed_identity=true sorted_windows_preserved=true \
             ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ie_runtime615_reused_chunk_hash_index_performance_evidence() {
    const HASHES: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;
    let hashes = (0..HASHES)
        .map(|index| {
            let mut hash = [0_u8; 32];
            hash[..8].copy_from_slice(&(index as u64).to_le_bytes());
            hash[8..].fill((index % 251) as u8);
            hash
        })
        .collect::<Vec<_>>();
    assert!(legacy_chunk_coverage(&hashes));
    assert!(reused_chunk_coverage(&hashes));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reused_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_chunk_coverage(black_box(&hashes)));
            legacy_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(reused_chunk_coverage(black_box(&hashes)));
            reused_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reused_chunk_coverage(black_box(&hashes)));
            reused_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(legacy_chunk_coverage(black_box(&hashes)));
            legacy_samples.push(started.elapsed());
        }
    }
    let legacy_p95 = percentile_95(&mut legacy_samples);
    let reused_p95 = percentile_95(&mut reused_samples);
    println!(
        "RUNTIME615_REUSED_CHUNK_HASH_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} chunk_hashes={HASHES} legacy_hash_sets=3 reused_hash_sets=2 legacy_p95_ns={} reused_p95_ns={} target_ratio_bp=7500",
        legacy_p95.as_nanos(),
        reused_p95.as_nanos(),
    );
    assert!(
        reused_p95.as_nanos() * 10_000 <= legacy_p95.as_nanos() * 7_500,
        "reused chunk-index P95 {:?} exceeded 75% of legacy P95 {:?}",
        reused_p95,
        legacy_p95,
    );
}
