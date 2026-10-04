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
    let source = include_str!("../../reader.rs");
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

fn legacy_read_asset(
    reader: &super::ZrPackReader,
    path: &str,
) -> Result<Vec<u8>, super::ZrPackError> {
    // Frozen pre-optimization read path: lookup, size/range validation,
    // per-read payload hash, then an owned copy.
    let asset = reader
        .manifest()
        .asset(path)
        .ok_or_else(|| super::ZrPackError::AssetNotFound(path.to_string()))?;
    let chunk = reader
        .manifest()
        .pack
        .chunks
        .binary_search_by_key(&asset.chunk_hash, |chunk| chunk.hash)
        .ok()
        .map(|index| &reader.manifest().pack.chunks[index])
        .ok_or_else(|| super::ZrPackError::MissingChunk(path.to_string()))?;
    if u64::from(chunk.size) != asset.size {
        return Err(super::ZrPackError::ChunkOutOfBounds(path.to_string()));
    }
    let bytes = super::chunk_range_bytes(&reader.bytes, chunk)
        .ok_or_else(|| super::ZrPackError::ChunkOutOfBounds(path.to_string()))?;
    if super::zrpack_content_hash(bytes) != chunk.hash {
        return Err(super::ZrPackError::ChunkHashMismatch(path.to_string()));
    }
    Ok(bytes.to_vec())
}

fn time_asset_reads(
    reader: &super::ZrPackReader,
    paths: &[&str],
    read: fn(&super::ZrPackReader, &str) -> Result<Vec<u8>, super::ZrPackError>,
) -> u128 {
    let started = Instant::now();
    for path in paths {
        black_box(read(black_box(reader), black_box(path)).unwrap());
    }
    started.elapsed().as_nanos()
}

fn read_percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = |percentile: usize| sorted[(sorted.len() * percentile).div_ceil(100) - 1];
    (rank(50), rank(95), rank(99))
}

#[test]
#[ignore = "Windows-native Release read-path performance evidence"]
fn runtime04_pack_reader_validated_read_reuse_bench() {
    use super::super::{ZrPackInputAsset, ZrPackWriter};
    use super::ZrPackReader;

    const ASSET_COUNT: usize = 64;
    const ASSET_BYTES: usize = 256 * 1_024;
    const READS_PER_SAMPLE: usize = 128;
    const WARMUP_PAIRS: usize = 5;
    const SAMPLE_PAIRS: usize = 31;
    assert!(!cfg!(debug_assertions), "run the read benchmark in Release");

    let paths = (0..ASSET_COUNT)
        .map(|index| format!("generated/read_{index:03}.bin"))
        .collect::<Vec<_>>();
    let assets = paths.iter().enumerate().map(|(index, path)| {
        let mut bytes = vec![index as u8; ASSET_BYTES];
        bytes[..8].copy_from_slice(&(index as u64).to_le_bytes());
        ZrPackInputAsset::new(path.clone(), bytes)
    });
    let pack = ZrPackWriter::write(assets).unwrap();
    let pack_hash = super::zrpack_content_hash(&pack.bytes);
    let reader = ZrPackReader::from_bytes(pack.bytes).unwrap();
    assert_eq!(reader.manifest().pack.chunks.len(), ASSET_COUNT);
    let read_paths = (0..READS_PER_SAMPLE)
        .map(|index| paths[(index * 37 + 11) % ASSET_COUNT].as_str())
        .collect::<Vec<_>>();

    // Validate every asset against the independent fixture and both paths
    // before measuring any lookup, hash, allocation, copy, or result drop.
    for (index, path) in paths.iter().enumerate() {
        let expected = {
            let mut bytes = vec![index as u8; ASSET_BYTES];
            bytes[..8].copy_from_slice(&(index as u64).to_le_bytes());
            bytes
        };
        let legacy = legacy_read_asset(&reader, path).unwrap();
        let optimized = reader.read_asset(path).unwrap();
        assert_eq!(legacy, expected);
        assert_eq!(optimized, expected);
        let expected_hash = reader.manifest().asset(path).unwrap().chunk_hash;
        assert_eq!(super::zrpack_content_hash(&legacy), expected_hash);
        assert_eq!(super::zrpack_content_hash(&optimized), expected_hash);
    }
    assert_eq!(
        legacy_read_asset(&reader, "missing.bin"),
        reader.read_asset("missing.bin")
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..(WARMUP_PAIRS + SAMPLE_PAIRS) {
        let (legacy, optimized) = if pair % 2 == 0 {
            (
                time_asset_reads(&reader, &read_paths, legacy_read_asset),
                time_asset_reads(&reader, &read_paths, ZrPackReader::read_asset),
            )
        } else {
            let optimized = time_asset_reads(&reader, &read_paths, ZrPackReader::read_asset);
            let legacy = time_asset_reads(&reader, &read_paths, legacy_read_asset);
            (legacy, optimized)
        };
        if pair >= WARMUP_PAIRS {
            legacy_samples.push(legacy);
            optimized_samples.push(optimized);
        }
    }
    let (legacy_p50, legacy_p95, legacy_p99) = read_percentiles(&legacy_samples);
    let (optimized_p50, optimized_p95, optimized_p99) = read_percentiles(&optimized_samples);
    println!(
        "RUNTIME04_PACK_READER_VALIDATED_READ_REUSE_BENCH_V1 assets={ASSET_COUNT} asset_bytes={ASSET_BYTES} reads_per_sample={READS_PER_SAMPLE} pack_hash={pack_hash:02x?} warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS} order=legacy_first_even_pair legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99} raw_legacy_ns={legacy_samples:?} raw_optimized_ns={optimized_samples:?} os={} arch={} package_version={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "verified immutable read P95 should be at most 95% of legacy read P95"
    );
}
