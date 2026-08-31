use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{sort_assets_by_path, ZrPackInputAsset, ZrPackWriter, FILE_READ_BUFFER_SIZE};

const ASSET_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 32;

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
    let source = include_str!("../writer.rs");
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
