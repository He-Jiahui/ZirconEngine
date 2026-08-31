use std::hint::black_box;
use std::time::Instant;

use super::super::ZrChunkEntry;
use super::{chunk_payload_end, sort_chunks_by_offset};

const CHUNK_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 64;

fn fixture_chunks() -> Vec<ZrChunkEntry> {
    (0..CHUNK_COUNT)
        .rev()
        .map(|index| ZrChunkEntry::new([index as u8; 32], 24 + (index as u64 * 16), 16))
        .collect()
}

fn legacy_sorted_offsets(chunks: &[ZrChunkEntry]) -> Vec<&ZrChunkEntry> {
    let mut sorted = chunks.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.offset.cmp(&right.offset));
    sorted
}

fn optimized_sorted_offsets(chunks: &[ZrChunkEntry]) -> Vec<&ZrChunkEntry> {
    let mut sorted = chunks.iter().collect::<Vec<_>>();
    sort_chunks_by_offset(&mut sorted);
    sorted
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_pack_reader_unstable_offset_sort_preserves_extent_order() {
    let chunks = fixture_chunks();
    let legacy = legacy_sorted_offsets(&chunks);
    let optimized = optimized_sorted_offsets(&chunks);

    assert_eq!(optimized, legacy);
    assert_eq!(chunk_payload_end(&chunks), Ok(24 + CHUNK_COUNT * 16));
}

#[test]
fn runtime04_pack_reader_unstable_offset_sort_source_contract() {
    let source = include_str!("../reader.rs");
    assert!(
        source.contains("chunks.sort_unstable_by(|left, right| left.offset.cmp(&right.offset))")
    );
    assert!(!source.contains("chunks.sort_by(|left, right| left.offset.cmp(&right.offset))"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_reader_unstable_offset_sort_bench() {
    let chunks = fixture_chunks();
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_sorted_offsets(&chunks));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(optimized_sorted_offsets(&chunks));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_PACK_READER_UNSTABLE_OFFSET_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} chunks={} stable_sorts=1->0",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        CHUNK_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized p95 should be at most 95% of legacy p95"
    );
}
