use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{sort_assets_by_path, ZrPackInputAsset, ZrPackWriter, FILE_READ_BUFFER_SIZE};

const ASSET_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 32;
const HASH_ASSET_COUNT: usize = 8_192;
const HASH_ITERATIONS: usize = 8;
const HASH_WARMUP_PAIRS: usize = 5;
const HASH_SAMPLE_PAIRS: usize = 31;

fn fixture_assets() -> Vec<ZrPackInputAsset> {
    (0..ASSET_COUNT)
        .rev()
        .map(|index| ZrPackInputAsset::new(format!("assets/{index:05}.bin"), [index as u8; 16]))
        .collect()
}

fn legacy_sorted_assets(mut assets: Vec<ZrPackInputAsset>) -> Vec<ZrPackInputAsset> {
    assets.sort_by(|left, right| left.path.cmp(&right.path));
    assets
}

fn optimized_sorted_assets(mut assets: Vec<ZrPackInputAsset>) -> Vec<ZrPackInputAsset> {
    sort_assets_by_path(&mut assets);
    assets
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn hash_admission_fixture() -> Vec<[u8; 32]> {
    let unique = (0..HASH_ASSET_COUNT)
        .map(|index| {
            let seed = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            let mut hash = [0_u8; 32];
            hash[..8].copy_from_slice(&seed.to_le_bytes());
            hash[8..16].copy_from_slice(&seed.rotate_left(17).to_le_bytes());
            hash[16..24].copy_from_slice(&seed.rotate_left(37).to_le_bytes());
            hash[24..].copy_from_slice(&seed.rotate_left(53).to_le_bytes());
            hash
        })
        .collect::<Vec<_>>();
    let mut hashes = Vec::with_capacity(HASH_ASSET_COUNT + HASH_ASSET_COUNT / 4);
    hashes.extend_from_slice(&unique);
    hashes.extend_from_slice(&unique[..HASH_ASSET_COUNT / 4]);
    hashes
}

fn legacy_hash_admissions(hashes: &[[u8; 32]]) -> usize {
    let mut offsets = BTreeMap::new();
    for (index, hash) in hashes.iter().enumerate() {
        if offsets.get(hash).is_none() {
            offsets.insert(*hash, index as u64);
        }
    }
    offsets.len()
}

fn single_lookup_hash_admissions(hashes: &[[u8; 32]]) -> usize {
    let mut hashes_seen = BTreeSet::new();
    for hash in hashes {
        hashes_seen.insert(*hash);
    }
    hashes_seen.len()
}

fn measure_hash_admissions(hashes: &[[u8; 32]], admit: fn(&[[u8; 32]]) -> usize) -> u128 {
    let started = Instant::now();
    for _ in 0..HASH_ITERATIONS {
        black_box(admit(black_box(hashes)));
    }
    started.elapsed().as_nanos()
}

fn hash_percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (rank(50), rank(95), rank(99))
}

#[test]
fn runtime04_pack_writer_unstable_path_sort_preserves_canonical_assets() {
    let legacy = legacy_sorted_assets(fixture_assets());
    let optimized = optimized_sorted_assets(fixture_assets());

    assert_eq!(optimized, legacy);
    assert!(optimized
        .windows(2)
        .all(|window| window[0].path <= window[1].path));
}

#[test]
fn runtime04_pack_writer_capacity_and_sort_source_contract() {
    let source = include_str!("../../writer.rs");
    assert!(source.contains(
        "assets.sort_unstable_by(|left, right| input_asset(left).path.cmp(&input_asset(right).path))"
    ));
    assert!(
        source.contains("chunk_entries.sort_unstable_by(|left, right| left.hash.cmp(&right.hash))")
    );
    assert!(source.contains("Vec::with_capacity(assets.len())"));
    assert!(!source.contains("assets.sort_by("));
}

#[test]
fn editor15_pack_writer_borrows_input_payloads_for_repeat_writes() {
    let assets = fixture_assets();
    let payload_addresses = assets
        .iter()
        .map(|asset| asset.bytes.as_ptr())
        .collect::<Vec<_>>();

    let first = ZrPackWriter::write(assets.iter()).expect("first borrowed write");
    let second = ZrPackWriter::write(assets.iter()).expect("second borrowed write");

    assert_eq!(first, second);
    assert_eq!(
        assets
            .iter()
            .map(|asset| asset.bytes.as_ptr())
            .collect::<Vec<_>>(),
        payload_addresses,
        "writer must not replace or consume the staged input payloads"
    );
}

#[test]
fn editor15_file_stream_writer_preserves_pack_bytes_and_deduplication() {
    let root = unique_temp_dir("file-stream-parity");
    let large_payload = vec![0x5a; FILE_READ_BUFFER_SIZE * 2 + 17];
    let distinct_payload = b"distinct payload".to_vec();
    let large_source = root.join("large.bin");
    let distinct_source = root.join("distinct.bin");
    let duplicate_source = root.join("duplicate.bin");
    fs::write(&large_source, &large_payload).expect("write large source");
    fs::write(&distinct_source, &distinct_payload).expect("write distinct source");
    fs::write(&duplicate_source, &large_payload).expect("write duplicate source");

    let memory_assets = vec![
        ZrPackInputAsset::new("assets/c.bin", large_payload.clone()),
        ZrPackInputAsset::new("assets/a.bin", large_payload),
        ZrPackInputAsset::new("assets/b.bin", distinct_payload),
    ];
    let memory_report = ZrPackWriter::write(memory_assets.iter()).expect("memory write");
    let file_report = ZrPackWriter::write_files([
        ("assets/c.bin", duplicate_source.as_path()),
        ("assets/a.bin", large_source.as_path()),
        ("assets/b.bin", distinct_source.as_path()),
    ])
    .expect("streamed file write");

    assert_eq!(file_report, memory_report);
    assert_eq!(file_report.deduplicated_assets, ["assets/c.bin"]);

    fs::remove_dir_all(root).expect("remove file stream fixture");
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_writer_unstable_path_sort_bench() {
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_sorted_assets(fixture_assets()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(optimized_sorted_assets(fixture_assets()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_PACK_WRITER_UNSTABLE_PATH_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} assets={} stable_sorts=2->0 reserved_slots=0->{}",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        ASSET_COUNT,
        ASSET_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_pack_writer_single_hash_membership_bench() {
    let hashes = hash_admission_fixture();
    assert_eq!(legacy_hash_admissions(&hashes), HASH_ASSET_COUNT);
    assert_eq!(single_lookup_hash_admissions(&hashes), HASH_ASSET_COUNT);
    let mut legacy_samples = Vec::with_capacity(HASH_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(HASH_SAMPLE_PAIRS);
    for pair in 0..(HASH_WARMUP_PAIRS + HASH_SAMPLE_PAIRS) {
        let (legacy_ns, optimized_ns) = if pair % 2 == 0 {
            (
                measure_hash_admissions(&hashes, legacy_hash_admissions),
                measure_hash_admissions(&hashes, single_lookup_hash_admissions),
            )
        } else {
            let optimized_ns = measure_hash_admissions(&hashes, single_lookup_hash_admissions);
            let legacy_ns = measure_hash_admissions(&hashes, legacy_hash_admissions);
            (legacy_ns, optimized_ns)
        };
        if pair < HASH_WARMUP_PAIRS {
            black_box((legacy_ns, optimized_ns));
            continue;
        }
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }
    let (legacy_p50, legacy_p95, legacy_p99) = hash_percentiles(&legacy_samples);
    let (optimized_p50, optimized_p95, optimized_p99) = hash_percentiles(&optimized_samples);
    println!(
        "RUNTIME04_PACK_WRITER_SINGLE_HASH_MEMBERSHIP_BENCH_V1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99} legacy_raw_ns={legacy_samples:?} optimized_raw_ns={optimized_samples:?} warmup_pairs={HASH_WARMUP_PAIRS} sample_pairs={HASH_SAMPLE_PAIRS} iterations={HASH_ITERATIONS} unique_hashes={HASH_ASSET_COUNT} duplicate_hashes={} pair_order=alternating_legacy_first_even unique_tree_lookups=2->1 os={} arch={} package_version={}",
        HASH_ASSET_COUNT / 4,
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "single hash membership p95 should be at most 95% of legacy"
    );
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-pack-writer-{label}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create pack writer fixture");
    root
}
