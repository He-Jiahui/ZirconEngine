use std::ffi::OsString;
use std::hint::black_box;
use std::io::{Seek, SeekFrom};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

#[test]
fn exact_inventory_membership_rejects_missing_replaced_and_extra_files() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-build-set-membership-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let nested = directory.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(directory.join("Cargo.toml"), b"[workspace]\n").unwrap();
    std::fs::write(nested.join("lib.rs"), b"pub fn fixture() {}\n").unwrap();

    let expected = ["Cargo.toml", "nested/lib.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    super::verify_snapshot_inventory(&directory, &expected).unwrap();

    let missing = ["Cargo.toml", "nested/lib.rs", "nested/missing.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(super::verify_snapshot_inventory(&directory, &missing).is_err());

    let replaced = ["Cargo.toml", "nested/other.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(super::verify_snapshot_inventory(&directory, &replaced).is_err());

    std::fs::write(nested.join("extra.rs"), b"pub fn extra() {}\n").unwrap();
    assert!(super::verify_snapshot_inventory(&directory, &expected).is_err());

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_non_boundary_unicode_timestamp_without_panicking() {
    let timestamp = format!("000{}{}Z", '\u{00e9}', "0".repeat(14));

    assert_eq!(timestamp.len(), 20);
    assert!(super::validate_utc_timestamp(&timestamp).is_err());
}

#[test]
fn recognizes_an_lfs_pointer_across_short_prefix_reads() {
    let pointer = b"version https://git-lfs.github.com/spec/v1\r\noid sha256:fixture";
    let mut prefix = [0_u8; super::GIT_LFS_PREFIX.len() + 2];
    let mut prefix_length = 0_usize;

    super::capture_prefix(&mut prefix, &mut prefix_length, &pointer[..13]);
    super::capture_prefix(&mut prefix, &mut prefix_length, &pointer[13..]);

    assert!(super::is_unmaterialized_lfs_pointer(
        &prefix[..prefix_length]
    ));
    assert!(!super::is_unmaterialized_lfs_pointer(super::GIT_LFS_PREFIX));
}

#[test]
fn stack_decimal_framing_matches_string_framing_at_u64_boundaries() {
    for value in [
        0,
        1,
        9,
        10,
        99,
        100,
        u32::MAX as u64,
        u64::MAX - 1,
        u64::MAX,
    ] {
        let mut baseline = Sha256::new();
        super::update_length_framed(&mut baseline, &value.to_string());
        let mut candidate = Sha256::new();
        super::update_length_framed_u64(&mut candidate, value);

        assert_eq!(baseline.finalize(), candidate.finalize(), "value={value}");
    }
}

#[test]
fn allocation_free_path_validation_preserves_path_safety_rules() {
    for valid in [
        "Cargo.toml",
        "crates/runtime/src/lib.rs",
        "assets/Unicode/\u{00e9}.txt",
    ] {
        assert!(super::validate_relative_path(valid).is_ok(), "{valid}");
        assert!(super::relative_path(valid).is_ok(), "{valid}");
    }
    for invalid in [
        "",
        " ",
        "/Cargo.toml",
        "Cargo.toml/",
        "Cargo.toml//nested",
        "src\\lib.rs",
        "./src/lib.rs",
        "src/../Cargo.toml",
        "C:/Cargo.toml",
        "C:Cargo.toml",
    ] {
        assert!(super::validate_relative_path(invalid).is_err(), "{invalid}");
        assert!(super::relative_path(invalid).is_err(), "{invalid}");
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn indexed_inventory_verification_performance_evidence() {
    const FILE_COUNT: usize = 100_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let observed = (0..FILE_COUNT)
        .map(|index| {
            let shuffled = index * 65_537 % FILE_COUNT;
            format!(
                "crates/runtime/src/shard_{:05}/module_{:06}/file_{:06}.rs",
                shuffled % 4_096,
                shuffled,
                FILE_COUNT - shuffled
            )
        })
        .collect::<Vec<_>>();
    let expected = observed
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_inventory_verification(&observed, &expected, round % 2 == 0);
        assert!(baseline.0 && candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_inventory_verification(&observed, &expected, round % 2 == 0);
        assert!(baseline.0 && candidate.0);
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
        "TOOLING15_BUILD_SET_INDEXED_INVENTORY_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_inventory_verification(
    observed: &[String],
    expected: &std::collections::HashSet<String>,
    baseline_first: bool,
) -> ((bool, Duration), (bool, Duration)) {
    let mut baseline = (false, Duration::ZERO);
    let mut candidate = (false, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        let mut current = observed.to_vec();
        current.sort_by(|left, right| super::ordinal_compare(left, right));
        baseline.0 =
            current.len() == expected.len() && current.iter().all(|path| expected.contains(path));
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let mut count = 0_usize;
        candidate.0 = observed.iter().all(|path| {
            let relative = black_box(path.clone());
            count += 1;
            expected.contains(&relative)
        }) && count == expected.len();
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[test]
#[ignore = "release-only performance evidence"]
fn fresh_handle_seek_elision_performance_evidence() {
    const FILE_COUNT: usize = 100_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-build-set-seek-bench-{}-{}.tmp",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    std::fs::write(&path, b"build-set").unwrap();
    let mut file = std::fs::File::open(&path).unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_seek_elision(&mut file, FILE_COUNT, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_seek_elision(&mut file, FILE_COUNT, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    drop(file);
    std::fs::remove_file(path).unwrap();

    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BUILD_SET_SEEK_ELISION_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn initial_metadata_reuse_performance_evidence() {
    const FILE_COUNT: usize = 10_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-build-set-metadata-bench-{}.tmp",
        std::process::id()
    ));
    std::fs::write(&path, b"build-set-metadata").unwrap();
    let file = std::fs::File::open(&path).unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_initial_metadata_reuse(&file, FILE_COUNT, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_initial_metadata_reuse(&file, FILE_COUNT, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    drop(file);
    std::fs::remove_file(path).unwrap();

    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BUILD_SET_METADATA_REUSE_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_initial_metadata_reuse(
    file: &std::fs::File,
    file_count: usize,
    baseline_first: bool,
) -> ((u64, Duration), (u64, Duration)) {
    let mut baseline = (0_u64, Duration::ZERO);
    let mut candidate = (0_u64, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        for _ in 0..file_count {
            black_box(file.metadata().unwrap());
            baseline.0 = baseline.0.wrapping_add(file.metadata().unwrap().len());
        }
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        for _ in 0..file_count {
            candidate.0 = candidate.0.wrapping_add(file.metadata().unwrap().len());
        }
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[test]
#[ignore = "release-only performance evidence"]
fn relative_path_buffer_performance_evidence() {
    const PATH_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 85;

    let paths = (0..PATH_COUNT)
        .map(|index| {
            std::path::PathBuf::from(format!(
                "crates/package_{:04}/src/generated/module_{index:06}.rs",
                index % 1_024
            ))
        })
        .collect::<Vec<_>>();
    let inventory_entries = paths
        .iter()
        .map(|path| {
            let relative = legacy_normalized_relative_text(path);
            let (directory, file_name) = relative.rsplit_once('/').unwrap();
            (directory.to_string(), OsString::from(file_name))
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_relative_paths(&paths, &inventory_entries, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_relative_paths(&paths, &inventory_entries, round % 2 == 0);
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
        "TOOLING15_BUILD_SET_RELATIVE_PATH_BUFFER_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 15%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 15%"
    );
}

fn measure_relative_paths(
    paths: &[std::path::PathBuf],
    inventory_entries: &[(String, OsString)],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        baseline.0 = paths
            .iter()
            .map(|path| legacy_normalized_relative_text(black_box(path)).len())
            .sum();
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        candidate.0 = inventory_entries
            .iter()
            .map(|(directory, file_name)| {
                super::snapshot_relative_path(
                    black_box(directory),
                    black_box(file_name.as_os_str()),
                )
                .unwrap()
                .len()
            })
            .sum();
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

fn legacy_normalized_relative_text(path: &std::path::Path) -> String {
    let mut normalized = String::new();
    for component in path.components() {
        let std::path::Component::Normal(component) = component else {
            panic!("benchmark fixture path must be normalized");
        };
        if !normalized.is_empty() {
            normalized.push('/');
        }
        normalized.push_str(component.to_str().unwrap());
    }
    normalized
}

fn measure_seek_elision(
    file: &mut std::fs::File,
    file_count: usize,
    baseline_first: bool,
) -> ((u64, Duration), (u64, Duration)) {
    let mut baseline = (0_u64, Duration::ZERO);
    let mut candidate = (0_u64, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        for _ in 0..file_count {
            baseline.0 ^= file.seek(SeekFrom::Start(0)).unwrap();
            baseline.0 ^= file.seek(SeekFrom::Start(0)).unwrap();
        }
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        for index in 0..file_count {
            candidate.0 ^= black_box(index as u64) & 0;
        }
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[test]
#[ignore = "release-only performance evidence"]
fn allocation_free_path_validation_performance_evidence() {
    const PATH_COUNT: usize = 100_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

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
        let (baseline, candidate) = measure_path_validation(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_path_validation(&paths, round % 2 == 0);
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
        "TOOLING15_BUILD_SET_PATH_VALIDATION_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_path_validation(
    paths: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        baseline.0 = paths
            .iter()
            .filter(|path| super::relative_path(black_box(path.as_str())).is_ok())
            .count();
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        candidate.0 = paths
            .iter()
            .filter(|path| super::validate_relative_path(black_box(path.as_str())).is_ok())
            .count();
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[test]
#[ignore = "release-only performance evidence"]
fn stack_decimal_framing_performance_evidence() {
    const FILE_COUNT: usize = 100_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let lengths = (0..FILE_COUNT)
        .map(|index| {
            (index as u64).wrapping_mul(1_000_003) ^ u64::MAX.rotate_right((index % 64) as u32)
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_length_framing(&lengths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_length_framing(&lengths, round % 2 == 0);
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
        "TOOLING15_BUILD_SET_DECIMAL_FRAME_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_length_framing(
    lengths: &[u64],
    baseline_first: bool,
) -> (([u8; 32], Duration), ([u8; 32], Duration)) {
    let mut baseline = ([0_u8; 32], Duration::ZERO);
    let mut candidate = ([0_u8; 32], Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        let mut hasher = Sha256::new();
        for value in lengths {
            super::update_length_framed(&mut hasher, &black_box(*value).to_string());
        }
        baseline = (hasher.finalize().into(), started.elapsed());
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let mut hasher = Sha256::new();
        for value in lengths {
            super::update_length_framed_u64(&mut hasher, black_box(*value));
        }
        candidate = (hasher.finalize().into(), started.elapsed());
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[test]
#[ignore = "release-only performance evidence"]
fn fixed_lfs_prefix_buffer_performance_evidence() {
    const FILE_COUNT: usize = 100_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let mut first_blocks = vec![[b'x'; 64]; FILE_COUNT];
    for (index, block) in first_blocks.iter_mut().enumerate() {
        block[56..64].copy_from_slice(&(index as u64).to_le_bytes());
        if index % 1_024 == 0 {
            block[..super::GIT_LFS_PREFIX.len()].copy_from_slice(super::GIT_LFS_PREFIX);
            block[super::GIT_LFS_PREFIX.len()] = b'\n';
        }
    }

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_lfs_prefix_scan(&first_blocks, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_lfs_prefix_scan(&first_blocks, round % 2 == 0);
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
        "TOOLING15_BUILD_SET_LFS_PREFIX_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_lfs_prefix_scan(
    first_blocks: &[[u8; 64]],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        baseline.0 = first_blocks
            .iter()
            .filter(|block| legacy_lfs_prefix_scan(black_box(block.as_slice())))
            .count();
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        candidate.0 = first_blocks
            .iter()
            .filter(|block| fixed_lfs_prefix_scan(black_box(block.as_slice())))
            .count();
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[inline(never)]
fn legacy_lfs_prefix_scan(block: &[u8]) -> bool {
    let mut prefix = Vec::with_capacity(super::GIT_LFS_PREFIX.len() + 2);
    for chunk in block.chunks(13) {
        if prefix.len() < prefix.capacity() {
            let remaining = prefix.capacity() - prefix.len();
            prefix.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        }
    }
    black_box(super::is_unmaterialized_lfs_pointer(&prefix))
}

#[inline(never)]
fn fixed_lfs_prefix_scan(block: &[u8]) -> bool {
    let mut prefix = [0_u8; super::GIT_LFS_PREFIX.len() + 2];
    let mut prefix_length = 0_usize;
    for chunk in block.chunks(13) {
        super::capture_prefix(&mut prefix, &mut prefix_length, chunk);
    }
    black_box(super::is_unmaterialized_lfs_pointer(
        &prefix[..prefix_length],
    ))
}

#[test]
#[ignore = "release-only performance evidence"]
fn ascii_ordinal_sort_performance_evidence() {
    const PATH_COUNT: usize = 100_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let paths = (0..PATH_COUNT)
        .map(|index| {
            let shuffled = index * 65_537 % PATH_COUNT;
            format!(
                "crates/runtime/src/shard_{:05}/module_{:06}/file_{:06}.rs",
                shuffled % 4_096,
                shuffled,
                PATH_COUNT - shuffled
            )
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_ascii_ordinal_sort(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_ascii_ordinal_sort(&paths, round % 2 == 0);
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
        "TOOLING15_BUILD_SET_ASCII_ORDINAL_BENCH_V1 baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_ascii_ordinal_sort<'a>(
    paths: &'a [String],
    baseline_first: bool,
) -> ((Vec<&'a str>, Duration), (Vec<&'a str>, Duration)) {
    let mut baseline = paths.iter().map(String::as_str).collect::<Vec<_>>();
    let mut candidate = baseline.clone();
    let mut baseline_duration = Duration::ZERO;
    let mut candidate_duration = Duration::ZERO;

    let mut run_baseline = || {
        let started = Instant::now();
        baseline.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
        baseline_duration = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        candidate.sort_by(|left, right| super::ordinal_compare(left, right));
        candidate_duration = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }

    (
        (baseline, baseline_duration),
        (candidate, candidate_duration),
    )
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
