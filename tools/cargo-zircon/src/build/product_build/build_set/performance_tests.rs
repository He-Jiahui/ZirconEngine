use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

const PATH_COUNT: usize = 20_000;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 75;
const DIRECT_PATHBUF_REQUIRED_PERCENT: u128 = 80;
const MANIFEST_READ_REPETITIONS: usize = 256;
const HANDLE_CLONE_REQUIRED_PERCENT: u128 = 90;
const HASH_BUFFER_FILE_COUNT: usize = 1_024;
const HASH_BUFFER_REUSE_REQUIRED_PERCENT: u128 = 70;
const RELATIVE_PATH_VALIDATION_REQUIRED_PERCENT: u128 = 80;
const TERMINAL_INVENTORY_FILE_COUNT: usize = 512;
const FOUR_PRODUCT_INVENTORY_PASSES: usize = 4;
const TERMINAL_INVENTORY_REQUIRED_PERCENT: u128 = 90;

#[test]
#[ignore = "release-only performance evidence"]
fn four_product_terminal_inventory_elision_performance_evidence() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-four-product-inventory-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut expected = HashSet::with_capacity(TERMINAL_INVENTORY_FILE_COUNT);
    for index in 0..TERMINAL_INVENTORY_FILE_COUNT {
        let relative = format!("shard_{:02}/file_{index:04}.rs", index % 16);
        let path = directory.join(super::relative_path(&relative).unwrap());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"pub fn fixture() {}\n").unwrap();
        expected.insert(relative);
    }

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_four_product_inventory(&directory, &expected, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_four_product_inventory(&directory, &expected, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    fs::remove_dir_all(directory).unwrap();

    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_FOUR_PRODUCT_TERMINAL_INVENTORY_ELISION_BENCH_V1 files={TERMINAL_INVENTORY_FILE_COUNT} baseline_passes={} candidate_passes={FOUR_PRODUCT_INVENTORY_PASSES} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        FOUR_PRODUCT_INVENTORY_PASSES + 1,
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * TERMINAL_INVENTORY_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * TERMINAL_INVENTORY_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_four_product_inventory(
    directory: &std::path::Path,
    expected: &HashSet<String>,
    baseline_first: bool,
) -> ((bool, Duration), (bool, Duration)) {
    let measure = |passes| {
        let started = Instant::now();
        let valid =
            (0..passes).all(|_| super::verify_snapshot_inventory(directory, expected).is_ok());
        (black_box(valid), started.elapsed())
    };
    if baseline_first {
        (
            measure(FOUR_PRODUCT_INVENTORY_PASSES + 1),
            measure(FOUR_PRODUCT_INVENTORY_PASSES),
        )
    } else {
        let candidate = measure(FOUR_PRODUCT_INVENTORY_PASSES);
        let baseline = measure(FOUR_PRODUCT_INVENTORY_PASSES + 1);
        (baseline, candidate)
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_pass_relative_path_validation_performance_evidence() {
    let paths = (0..PATH_COUNT)
        .map(|index| {
            format!(
                "crates/runtime/src/shard_{:05}/module_{:06}/file_{:06}.rs",
                index % 4_096,
                index,
                PATH_COUNT - index
            )
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_relative_path_validation(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_relative_path_validation(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BUILD_SET_SINGLE_PASS_RELATIVE_PATH_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * RELATIVE_PATH_VALIDATION_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * RELATIVE_PATH_VALIDATION_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

fn measure_relative_path_validation(
    paths: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let valid = paths
            .iter()
            .filter(|path| legacy_relative_path_is_valid(black_box(path.as_str())))
            .count();
        (black_box(valid), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let valid = paths
            .iter()
            .filter(|path| super::validate_relative_path(black_box(path.as_str())).is_ok())
            .count();
        (black_box(valid), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn legacy_relative_path_is_valid(value: &str) -> bool {
    use std::path::{Component, Path};

    !value.trim().is_empty()
        && !value.contains('\\')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !Path::new(value)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
}

#[test]
#[ignore = "release-only performance evidence"]
fn reused_path_buffer_performance_evidence() {
    let entries = path_entries();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_path_construction(&entries, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_path_construction(&entries, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BUILD_SET_REUSED_PATH_BUFFER_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 25%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 25%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn direct_relative_pathbuf_performance_evidence() {
    let paths = (0..PATH_COUNT)
        .map(|index| {
            format!(
                "crates/package_{:04}/src/module_{index:06}.rs",
                index % 1_024
            )
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_relative_pathbuf(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_relative_pathbuf(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BUILD_SET_DIRECT_PATHBUF_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * DIRECT_PATHBUF_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * DIRECT_PATHBUF_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn manifest_handle_clone_elision_performance_evidence() {
    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-manifest-read-benchmark-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, vec![b'X'; 4_096]).unwrap();
    let mut file = File::open(&path).unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_manifest_reads(&mut file, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_manifest_reads(&mut file, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);
    drop(file);
    std::fs::remove_file(path).unwrap();

    println!(
        "TOOLING15_BUILD_SET_HANDLE_CLONE_ELISION_BENCH_V1 reads={MANIFEST_READ_REPETITIONS} bytes=4096 rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * HANDLE_CLONE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * HANDLE_CLONE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn shared_hash_buffer_performance_evidence() {
    let payloads = (0..HASH_BUFFER_FILE_COUNT)
        .map(|index| vec![(index % 251) as u8; 256])
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_hash_buffers(&payloads, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_hash_buffers(&payloads, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BUILD_SET_HASH_BUFFER_REUSE_BENCH_V1 files={HASH_BUFFER_FILE_COUNT} bytes_per_file=256 rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * HASH_BUFFER_REUSE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * HASH_BUFFER_REUSE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_hash_buffers(
    payloads: &[Vec<u8>],
    baseline_first: bool,
) -> ((u8, Duration), (u8, Duration)) {
    if baseline_first {
        let baseline = measure_per_file_hash_buffers(payloads);
        let candidate = measure_shared_hash_buffer(payloads);
        (baseline, candidate)
    } else {
        let candidate = measure_shared_hash_buffer(payloads);
        let baseline = measure_per_file_hash_buffers(payloads);
        (baseline, candidate)
    }
}

fn measure_per_file_hash_buffers(payloads: &[Vec<u8>]) -> (u8, Duration) {
    let started = Instant::now();
    let mut checksum = 0_u8;
    for payload in payloads {
        let mut buffer = [0_u8; super::BUILD_SET_HASH_BUFFER_BYTES];
        let count = Cursor::new(black_box(payload.as_slice()))
            .read(black_box(&mut buffer[..]))
            .unwrap();
        checksum ^= Sha256::digest(&buffer[..count])[0];
    }
    (black_box(checksum), started.elapsed())
}

fn measure_shared_hash_buffer(payloads: &[Vec<u8>]) -> (u8, Duration) {
    let started = Instant::now();
    let mut checksum = 0_u8;
    let mut buffer = [0_u8; super::BUILD_SET_HASH_BUFFER_BYTES];
    for payload in payloads {
        let count = Cursor::new(black_box(payload.as_slice()))
            .read(black_box(&mut buffer[..]))
            .unwrap();
        checksum ^= Sha256::digest(&buffer[..count])[0];
    }
    (black_box(checksum), started.elapsed())
}

fn measure_manifest_reads(
    file: &mut File,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_legacy_manifest_reads(file);
        let candidate = measure_direct_manifest_reads(file);
        (baseline, candidate)
    } else {
        let candidate = measure_direct_manifest_reads(file);
        let baseline = measure_legacy_manifest_reads(file);
        (baseline, candidate)
    }
}

fn measure_legacy_manifest_reads(file: &mut File) -> (usize, Duration) {
    let started = Instant::now();
    let mut byte_count = 0_usize;
    for _ in 0..MANIFEST_READ_REPETITIONS {
        file.rewind().unwrap();
        let bytes = legacy_read_bounded_file(file, 8_192);
        byte_count = byte_count.saturating_add(black_box(bytes.len()));
    }
    (black_box(byte_count), started.elapsed())
}

fn measure_direct_manifest_reads(file: &mut File) -> (usize, Duration) {
    let started = Instant::now();
    let mut byte_count = 0_usize;
    for _ in 0..MANIFEST_READ_REPETITIONS {
        file.rewind().unwrap();
        let bytes = super::read_bounded_file(file, 8_192, "benchmark manifest").unwrap();
        byte_count = byte_count.saturating_add(black_box(bytes.len()));
    }
    (black_box(byte_count), started.elapsed())
}

fn legacy_read_bounded_file(file: &File, limit: usize) -> Vec<u8> {
    let declared_length = file.metadata().unwrap().len();
    let reader = file.try_clone().unwrap();
    let mut bytes = Vec::with_capacity(declared_length as usize);
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

fn measure_relative_pathbuf(
    paths: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || measure_pathbuf_components(paths, legacy_relative_path);
    let measure_candidate =
        || measure_pathbuf_components(paths, |value| super::relative_path(value).unwrap());
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_pathbuf_components(
    paths: &[String],
    construct: impl Fn(&str) -> PathBuf,
) -> (usize, Duration) {
    let started = Instant::now();
    let mut component_count = 0_usize;
    for value in paths {
        let path = construct(black_box(value.as_str()));
        component_count = component_count.saturating_add(black_box(path.components().count()));
    }
    (black_box(component_count), started.elapsed())
}

fn legacy_relative_path(value: &str) -> PathBuf {
    super::validate_relative_path(value).unwrap();
    value.split('/').collect()
}

fn path_entries() -> Vec<(String, OsString)> {
    (0..PATH_COUNT)
        .map(|index| {
            (
                format!("crates/package_{:04}/src/generated", index % 1_024),
                OsString::from(format!("module_{index:06}.rs")),
            )
        })
        .collect()
}

fn measure_path_construction(
    entries: &[(String, OsString)],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_owned_paths(entries);
        let candidate = measure_reused_buffer(entries);
        (baseline, candidate)
    } else {
        let candidate = measure_reused_buffer(entries);
        let baseline = measure_owned_paths(entries);
        (baseline, candidate)
    }
}

fn measure_owned_paths(entries: &[(String, OsString)]) -> (usize, Duration) {
    let started = Instant::now();
    let mut total_length = 0_usize;
    for (directory, file_name) in entries {
        let relative =
            super::snapshot_relative_path(black_box(directory), black_box(file_name.as_os_str()))
                .unwrap();
        total_length = total_length.saturating_add(black_box(relative.len()));
    }
    (black_box(total_length), started.elapsed())
}

fn measure_reused_buffer(entries: &[(String, OsString)]) -> (usize, Duration) {
    let started = Instant::now();
    let mut total_length = 0_usize;
    let mut relative = String::new();
    for (directory, file_name) in entries {
        super::snapshot_relative_path_into(
            &mut relative,
            black_box(directory),
            black_box(file_name.as_os_str()),
        )
        .unwrap();
        total_length = total_length.saturating_add(black_box(relative.len()));
    }
    (black_box(total_length), started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
