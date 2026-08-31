use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{zrpack_content_hash, ZrPackDedupTable};

const CHUNK_COUNT: usize = 4_096;
const UNIQUE_CHUNKS: usize = 512;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 16;

fn fixture_chunks() -> Vec<Vec<u8>> {
    (0..CHUNK_COUNT)
        .map(|index| {
            let mut bytes = vec![0u8; 512];
            bytes[..8].copy_from_slice(&(index % UNIQUE_CHUNKS as usize).to_le_bytes());
            bytes
        })
        .collect()
}

fn legacy_insert_or_get(
    chunks: &mut BTreeMap<[u8; 32], usize>,
    bytes: &[u8],
) -> ([u8; 32], Option<usize>) {
    let hash = zrpack_content_hash(bytes);
    let existing = chunks.get(&hash).copied();
    if existing.is_none() {
        let index = chunks.len();
        chunks.insert(hash, index);
    }
    (hash, existing)
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_pack_dedup_entry_lookup_preserves_first_indices() {
    let chunks = fixture_chunks();
    let mut legacy = BTreeMap::new();
    let mut optimized = ZrPackDedupTable::default();
    for bytes in &chunks {
        assert_eq!(
            optimized.insert_or_get(bytes),
            legacy_insert_or_get(&mut legacy, bytes)
        );
    }
    assert_eq!(optimized.len(), UNIQUE_CHUNKS);
    assert_eq!(optimized.len(), legacy.len());
}

#[test]
fn runtime04_pack_dedup_entry_lookup_source_contract() {
    let source = include_str!("../dedup.rs");
    assert!(source.contains("match self.chunks.entry(hash)"));
    assert!(source.contains("Entry::Occupied(entry)"));
    assert!(source.contains("Entry::Vacant(entry)"));
    assert!(!source.contains("self.chunks.get(&hash).copied()"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_dedup_entry_lookup_bench() {
    let chunks = fixture_chunks();
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut table = BTreeMap::new();
                for bytes in &chunks {
                    black_box(legacy_insert_or_get(&mut table, bytes));
                }
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut table = ZrPackDedupTable::default();
                for bytes in &chunks {
                    black_box(table.insert_or_get(bytes));
                }
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_PACK_DEDUP_ENTRY_LOOKUP_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} chunks={} unique_chunks={} btree_lookups=2->1",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        CHUNK_COUNT,
        UNIQUE_CHUNKS,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized p95 should be at most 95% of legacy p95"
    );
}
