use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{
    days_in_month, validate_created_utc, validate_normalized_artifact_closure,
    validate_sha256_if_normalized, validate_unique_artifact_names,
    validate_unique_sorted_artifact_names,
};
use crate::build::receipt::{ArtifactKind, ReceiptArtifact};

const FEATURE_COUNT: usize = 20_000;
const ARTIFACT_COUNT: usize = 50_000;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 40;
const ARTIFACT_MERGE_REQUIRED_PERCENT: u128 = 70;
const FUSED_ARTIFACT_UNIQUENESS_REQUIRED_PERCENT: u128 = 90;
const NORMALIZED_ARTIFACT_CLOSURE_REQUIRED_PERCENT: u128 = 90;
const UNSTABLE_SORT_REQUIRED_PERCENT: u128 = 90;
const IDENTITY_RECORD_SORT_REQUIRED_PERCENT: u128 = 90;
const SHA256_COUNT: usize = 16_384;
const SHA256_REQUIRED_PERCENT: u128 = 80;
const TIMESTAMP_COUNT: usize = 16_384;
const FIXED_TIMESTAMP_REQUIRED_PERCENT: u128 = 75;
const RELATIVE_PATH_COUNT: usize = 32_768;
const RELATIVE_PATH_REQUIRED_PERCENT: u128 = 80;

#[test]
#[ignore = "release-only performance evidence"]
fn single_pass_relative_path_performance_evidence() {
    let paths = (0..RELATIVE_PATH_COUNT)
        .map(|index| {
            format!(
                "runtime/shard_{:04}/platform/win64/artifact_{index:05}.dll",
                index % 1_024
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
        "TOOLING15_SINGLE_PASS_RELATIVE_PATH_BENCH_V1 paths={RELATIVE_PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * RELATIVE_PATH_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * RELATIVE_PATH_REQUIRED_PERCENT,
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

    !value.is_empty()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains("//")
        && !value.contains('\\')
        && !Path::new(value).is_absolute()
        && !Path::new(value).components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_pass_sha256_validation_performance_evidence() {
    let digests = (0..SHA256_COUNT)
        .map(|index| format!("{index:016X}").repeat(4))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_sha256_validation(&digests, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_sha256_validation(&digests, round % 2 == 0);
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
        "TOOLING15_SINGLE_PASS_SHA256_VALIDATION_BENCH_V1 digests={SHA256_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * SHA256_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * SHA256_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

fn measure_sha256_validation(
    digests: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let normalized = digests
            .iter()
            .filter(|digest| {
                let digest = black_box(digest.as_str());
                digest.len() == 64
                    && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                    && !digest.bytes().any(|byte| byte.is_ascii_lowercase())
            })
            .count();
        (black_box(normalized), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let normalized = digests
            .iter()
            .filter(|digest| {
                validate_sha256_if_normalized("benchmark", black_box(digest.as_str())).unwrap()
            })
            .count();
        (black_box(normalized), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn fixed_layout_timestamp_performance_evidence() {
    let timestamps = (0..TIMESTAMP_COUNT)
        .map(|index| format!("2026-08-29T12:34:56.{index:06}Z"))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_timestamp_validation(&timestamps, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_timestamp_validation(&timestamps, round % 2 == 0);
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
        "TOOLING15_FIXED_LAYOUT_TIMESTAMP_BENCH_V1 timestamps={TIMESTAMP_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * FIXED_TIMESTAMP_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 25%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * FIXED_TIMESTAMP_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 25%"
    );
}

fn measure_timestamp_validation(
    timestamps: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let valid = timestamps
            .iter()
            .filter(|value| legacy_created_utc_is_valid(black_box(value.as_str())))
            .count();
        (black_box(valid), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let valid = timestamps
            .iter()
            .filter(|value| validate_created_utc(black_box(value.as_str())).is_ok())
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

fn legacy_created_utc_is_valid(value: &str) -> bool {
    if !value.is_ascii() {
        return false;
    }
    let Some(timestamp) = value.strip_suffix('Z') else {
        return false;
    };
    let (base, fraction) = match timestamp.split_once('.') {
        Some((base, fraction)) => (base, Some(fraction)),
        None => (timestamp, None),
    };
    if base.len() != 19
        || &base[4..5] != "-"
        || &base[7..8] != "-"
        || &base[10..11] != "T"
        || &base[13..14] != ":"
        || &base[16..17] != ":"
        || !base[0..4].bytes().all(|byte| byte.is_ascii_digit())
        || !base[5..7].bytes().all(|byte| byte.is_ascii_digit())
        || !base[8..10].bytes().all(|byte| byte.is_ascii_digit())
        || !base[11..13].bytes().all(|byte| byte.is_ascii_digit())
        || !base[14..16].bytes().all(|byte| byte.is_ascii_digit())
        || !base[17..19].bytes().all(|byte| byte.is_ascii_digit())
        || fraction.is_some_and(|value| {
            value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return false;
    }

    let decimal = |value: &str| {
        value
            .bytes()
            .fold(0_u32, |total, byte| total * 10 + u32::from(byte - b'0'))
    };
    let year = decimal(&base[0..4]);
    let month = decimal(&base[5..7]);
    let day = decimal(&base[8..10]);
    let hour = decimal(&base[11..13]);
    let minute = decimal(&base[14..16]);
    let second = decimal(&base[17..19]);
    month != 0
        && month <= 12
        && day != 0
        && day <= days_in_month(year, month)
        && hour <= 23
        && minute <= 59
        && second <= 60
}

#[test]
#[ignore = "release-only performance evidence"]
fn sorted_feature_dedup_performance_evidence() {
    let features = (0..FEATURE_COUNT)
        .map(|index| format!("product-feature-{index:05}"))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_dedup(&features, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_dedup(&features, round % 2 == 0);
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
        "TOOLING15_SORTED_FEATURE_DEDUP_BENCH_V1 features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 60%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 60%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn sorted_artifact_name_merge_performance_evidence() {
    let (build_products, runtime_dependencies, symbols, sbom) = benchmark_artifacts();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_artifact_names(
            &build_products,
            &runtime_dependencies,
            &symbols,
            &sbom,
            round % 2 == 0,
        );
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_artifact_names(
            &build_products,
            &runtime_dependencies,
            &symbols,
            &sbom,
            round % 2 == 0,
        );
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
        "TOOLING15_SORTED_ARTIFACT_NAME_MERGE_BENCH_V1 artifacts={} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        ARTIFACT_COUNT + 1,
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * ARTIFACT_MERGE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * ARTIFACT_MERGE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn fused_artifact_uniqueness_performance_evidence() {
    let (build_products, runtime_dependencies, symbols, sbom) = benchmark_artifacts();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_artifact_uniqueness(
            &build_products,
            &runtime_dependencies,
            &symbols,
            &sbom,
            round % 2 == 0,
        );
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_artifact_uniqueness(
            &build_products,
            &runtime_dependencies,
            &symbols,
            &sbom,
            round % 2 == 0,
        );
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
        "TOOLING15_FUSED_ARTIFACT_UNIQUENESS_BENCH_V1 artifacts={} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        ARTIFACT_COUNT + 1,
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * FUSED_ARTIFACT_UNIQUENESS_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * FUSED_ARTIFACT_UNIQUENESS_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn normalized_artifact_closure_performance_evidence() {
    let (build_products, runtime_dependencies, symbols, sbom) = benchmark_artifacts();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_normalized_artifact_closure(
            &build_products,
            &runtime_dependencies,
            &symbols,
            &sbom,
            round % 2 == 0,
        );
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_normalized_artifact_closure(
            &build_products,
            &runtime_dependencies,
            &symbols,
            &sbom,
            round % 2 == 0,
        );
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
        "TOOLING15_NORMALIZED_ARTIFACT_CLOSURE_BENCH_V1 artifacts={} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        ARTIFACT_COUNT + 1,
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * NORMALIZED_ARTIFACT_CLOSURE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * NORMALIZED_ARTIFACT_CLOSURE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn unstable_feature_sort_performance_evidence() {
    let features = (0..FEATURE_COUNT)
        .map(|index| format!("product-feature-{:05}", (index * 7_919) % FEATURE_COUNT))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_feature_sort(&features, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_feature_sort(&features, round % 2 == 0);
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
        "TOOLING15_UNSTABLE_FEATURE_SORT_BENCH_V1 features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * UNSTABLE_SORT_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * UNSTABLE_SORT_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn unstable_identity_record_sort_performance_evidence() {
    let (mut artifacts, runtime_dependencies, symbols, sbom) = benchmark_artifacts();
    artifacts.extend(runtime_dependencies);
    artifacts.extend(symbols);
    artifacts.push(sbom);
    artifacts.reverse();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_identity_record_sort(&artifacts, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_identity_record_sort(&artifacts, round % 2 == 0);
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
        "TOOLING15_UNSTABLE_IDENTITY_RECORD_SORT_BENCH_V1 artifacts={} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        artifacts.len(),
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * IDENTITY_RECORD_SORT_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * IDENTITY_RECORD_SORT_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_identity_record_sort(
    artifacts: &[ReceiptArtifact],
    baseline_first: bool,
) -> (
    (Vec<ReceiptArtifact>, Duration),
    (Vec<ReceiptArtifact>, Duration),
) {
    let measure_baseline = || {
        let mut values = artifacts.to_vec();
        let started = Instant::now();
        values.sort_by(|left, right| left.logical_name.cmp(&right.logical_name));
        (black_box(values), started.elapsed())
    };
    let measure_candidate = || {
        let mut values = artifacts.to_vec();
        let started = Instant::now();
        values.sort_unstable_by(|left, right| left.logical_name.cmp(&right.logical_name));
        (black_box(values), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_feature_sort(
    features: &[String],
    baseline_first: bool,
) -> ((Vec<String>, Duration), (Vec<String>, Duration)) {
    if baseline_first {
        let baseline = measure_stable_sort(features);
        let candidate = measure_unstable_sort(features);
        (baseline, candidate)
    } else {
        let candidate = measure_unstable_sort(features);
        let baseline = measure_stable_sort(features);
        (baseline, candidate)
    }
}

fn measure_stable_sort(features: &[String]) -> (Vec<String>, Duration) {
    let mut values = features.to_vec();
    let started = Instant::now();
    values.sort();
    let elapsed = started.elapsed();
    (black_box(values), elapsed)
}

fn measure_unstable_sort(features: &[String]) -> (Vec<String>, Duration) {
    let mut values = features.to_vec();
    let started = Instant::now();
    values.sort_unstable();
    let elapsed = started.elapsed();
    (black_box(values), elapsed)
}

fn measure_artifact_names(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: &ReceiptArtifact,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_name_hash_set(build_products, runtime_dependencies, symbols, sbom);
        let candidate = measure_name_merge(build_products, runtime_dependencies, symbols, sbom);
        (baseline, candidate)
    } else {
        let candidate = measure_name_merge(build_products, runtime_dependencies, symbols, sbom);
        let baseline = measure_name_hash_set(build_products, runtime_dependencies, symbols, sbom);
        (baseline, candidate)
    }
}

fn measure_artifact_uniqueness(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: &ReceiptArtifact,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        validate_unique_sorted_artifact_names(
            black_box(build_products),
            black_box(runtime_dependencies),
            black_box(symbols),
            Some(black_box(sbom)),
        )
        .unwrap();
        let mut paths = HashSet::with_capacity(ARTIFACT_COUNT + 1);
        for artifact in black_box(build_products)
            .iter()
            .chain(black_box(runtime_dependencies))
            .chain(black_box(symbols))
            .chain(Some(black_box(sbom)))
        {
            assert!(paths.insert(artifact.relative_path.as_str()));
        }
        (black_box(paths.len()), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        validate_unique_artifact_names(
            black_box(build_products),
            black_box(runtime_dependencies),
            black_box(symbols),
            Some(black_box(sbom)),
        )
        .unwrap();
        (black_box(ARTIFACT_COUNT + 1), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_normalized_artifact_closure(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: &ReceiptArtifact,
    baseline_first: bool,
) -> ((bool, Duration), (bool, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let fields = validate_artifacts_if_normalized_for_benchmark(
            build_products,
            runtime_dependencies,
            symbols,
            sbom,
        )
        .unwrap();
        validate_unique_artifact_names(build_products, runtime_dependencies, symbols, Some(sbom))
            .unwrap();
        (black_box(fields), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let result = validate_normalized_artifact_closure(
            black_box(build_products),
            black_box(runtime_dependencies),
            black_box(symbols),
            Some(black_box(sbom)),
        )
        .unwrap();
        (black_box(result), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn validate_artifacts_if_normalized_for_benchmark(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: &ReceiptArtifact,
) -> Result<bool, super::ProductReceiptError> {
    let partitions: [(&[ReceiptArtifact], &str, fn(&ArtifactKind) -> bool); 3] = [
        (build_products, "build product", |kind| {
            matches!(kind, ArtifactKind::Executable)
        }),
        (runtime_dependencies, "runtime dependency", |kind| {
            matches!(kind, ArtifactKind::DynamicLibrary | ArtifactKind::Resource)
        }),
        (symbols, "symbol", |kind| {
            matches!(kind, ArtifactKind::SymbolFile)
        }),
    ];
    for (artifacts, partition, allowed_kind) in partitions {
        let mut previous: Option<&str> = None;
        for artifact in artifacts {
            if previous.is_some_and(|name| name > artifact.logical_name.as_str()) {
                return Ok(false);
            }
            if !super::validate_artifact_if_normalized(artifact)? {
                return Ok(false);
            }
            super::validate_artifact_kind(artifact, partition, allowed_kind)?;
            previous = Some(artifact.logical_name.as_str());
        }
    }
    if !super::validate_artifact_if_normalized(sbom)? {
        return Ok(false);
    }
    super::validate_artifact_kind(sbom, "SBOM", |kind| matches!(kind, ArtifactKind::Sbom))?;
    Ok(true)
}

fn measure_name_hash_set(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: &ReceiptArtifact,
) -> (usize, Duration) {
    let started = Instant::now();
    let mut names = HashSet::with_capacity(ARTIFACT_COUNT + 1);
    for artifact in black_box(build_products)
        .iter()
        .chain(black_box(runtime_dependencies))
        .chain(black_box(symbols))
        .chain(Some(black_box(sbom)))
    {
        assert!(names.insert(artifact.logical_name.as_str()));
    }
    (black_box(names.len()), started.elapsed())
}

fn measure_name_merge(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: &ReceiptArtifact,
) -> (usize, Duration) {
    let started = Instant::now();
    validate_unique_sorted_artifact_names(
        black_box(build_products),
        black_box(runtime_dependencies),
        black_box(symbols),
        Some(black_box(sbom)),
    )
    .unwrap();
    (black_box(ARTIFACT_COUNT + 1), started.elapsed())
}

fn benchmark_artifacts() -> (
    Vec<ReceiptArtifact>,
    Vec<ReceiptArtifact>,
    Vec<ReceiptArtifact>,
    ReceiptArtifact,
) {
    let mut partitions = [Vec::new(), Vec::new(), Vec::new()];
    for index in 0..ARTIFACT_COUNT {
        let partition_index = index % 3;
        partitions[partition_index].push(ReceiptArtifact {
            logical_name: format!("artifact-{index:05}"),
            relative_path: format!("partition-{}/artifact-{index:05}.bin", index % 3),
            kind: match partition_index {
                0 => ArtifactKind::Executable,
                1 => ArtifactKind::DynamicLibrary,
                _ => ArtifactKind::SymbolFile,
            },
            sha256: "A".repeat(64),
            byte_length: 4_096,
        });
    }
    let [build_products, runtime_dependencies, symbols] = partitions;
    let sbom = ReceiptArtifact {
        logical_name: "zzzz-product-sbom".to_string(),
        relative_path: "sbom/product.spdx.json".to_string(),
        kind: ArtifactKind::Sbom,
        sha256: "B".repeat(64),
        byte_length: 4_096,
    };
    (build_products, runtime_dependencies, symbols, sbom)
}

fn measure_dedup(
    features: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_hash_set(features);
        let candidate = measure_adjacent(features);
        (baseline, candidate)
    } else {
        let candidate = measure_adjacent(features);
        let baseline = measure_hash_set(features);
        (baseline, candidate)
    }
}

fn measure_hash_set(features: &[String]) -> (usize, Duration) {
    let started = Instant::now();
    let mut unique = HashSet::new();
    for feature in black_box(features) {
        assert!(unique.insert(feature.as_str()));
    }
    (black_box(unique.len()), started.elapsed())
}

fn measure_adjacent(features: &[String]) -> (usize, Duration) {
    let started = Instant::now();
    let mut previous = None;
    let mut count = 0;
    for feature in black_box(features) {
        assert_ne!(previous, Some(feature.as_str()));
        previous = Some(feature.as_str());
        count += 1;
    }
    (black_box(count), started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
