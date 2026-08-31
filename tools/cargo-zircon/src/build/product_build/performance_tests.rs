use std::ffi::{OsStr, OsString};
use std::hint::black_box;
use std::io::{Cursor, Read};
use std::path::Path;
use std::time::{Duration, Instant};

use super::ProductBuildRequest;

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn borrowed_cargo_arguments_performance_evidence() {
    const CONSTRUCTION_COUNT: usize = 10_000;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 80;

    let mut request = super::tests::test_request();
    request.action.features.clear();
    let manifest_path = Path::new("snapshot/Cargo.toml");
    let target_directory = Path::new("output/target");
    let binary = request.action.bin.as_deref().unwrap();

    let baseline_metadata = legacy_metadata_arguments(&request, manifest_path);
    let candidate_metadata = super::metadata_arguments(&request, manifest_path);
    assert_eq!(
        argument_text(&baseline_metadata),
        argument_text(&candidate_metadata)
    );
    let baseline_arguments =
        legacy_build_arguments(&request, manifest_path, target_directory, binary);
    let candidate_arguments =
        super::build_arguments(&request, manifest_path, target_directory, binary);
    assert_eq!(
        argument_text(&baseline_arguments),
        argument_text(&candidate_arguments)
    );

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_argument_construction(
            &request,
            manifest_path,
            target_directory,
            binary,
            CONSTRUCTION_COUNT,
            round % 2 == 0,
        );
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_argument_construction(
            &request,
            manifest_path,
            target_directory,
            binary,
            CONSTRUCTION_COUNT,
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
        "TOOLING15_BORROWED_CARGO_ARGUMENTS_BENCH_V1 constructions={CONSTRUCTION_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn bounded_cargo_output_buffer_performance_evidence() {
    const OUTPUT_BYTES: usize = 768 * 1024;
    const OUTPUT_LIMIT: usize = 8 * 1024 * 1024;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 90;

    let payload = vec![b'x'; OUTPUT_BYTES];
    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_bounded_output_read(&payload, OUTPUT_LIMIT, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_bounded_output_read(&payload, OUTPUT_LIMIT, round % 2 == 0);
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
        "TOOLING15_BOUNDED_CARGO_OUTPUT_BUFFER_BENCH_V1 bytes={OUTPUT_BYTES} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_bounded_output_read(
    payload: &[u8],
    limit: usize,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        let mut output = Vec::new();
        Cursor::new(black_box(payload))
            .take(limit as u64 + 1)
            .read_to_end(&mut output)
            .unwrap();
        baseline = (black_box(output.len()), started.elapsed());
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let mut output = super::bounded_output_buffer(limit);
        Cursor::new(black_box(payload))
            .take(limit as u64 + 1)
            .read_to_end(&mut output)
            .unwrap();
        candidate = (black_box(output.len()), started.elapsed());
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

fn legacy_metadata_arguments(request: &ProductBuildRequest, manifest_path: &Path) -> Vec<OsString> {
    vec![
        "metadata".into(),
        "--format-version".into(),
        "1".into(),
        "--manifest-path".into(),
        manifest_path.as_os_str().to_owned(),
        "--frozen".into(),
        "--filter-platform".into(),
        request.target.target_triple.clone().into(),
    ]
}

fn legacy_build_arguments(
    request: &ProductBuildRequest,
    manifest_path: &Path,
    target_directory: &Path,
    binary: &str,
) -> Vec<OsString> {
    vec![
        "build".into(),
        "--manifest-path".into(),
        manifest_path.as_os_str().to_owned(),
        "--package".into(),
        request.action.package.clone().into(),
        "--bin".into(),
        binary.into(),
        "--frozen".into(),
        "--target".into(),
        request.target.target_triple.clone().into(),
        "--profile".into(),
        request.target.cargo_profile.clone().into(),
        "--target-dir".into(),
        target_directory.as_os_str().to_owned(),
        "--message-format=json-render-diagnostics".into(),
    ]
}

fn measure_argument_construction(
    request: &ProductBuildRequest,
    manifest_path: &Path,
    target_directory: &Path,
    binary: &str,
    construction_count: usize,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        baseline.0 = (0..construction_count)
            .map(|_| {
                let metadata =
                    legacy_metadata_arguments(black_box(request), black_box(manifest_path));
                let build = legacy_build_arguments(
                    black_box(request),
                    black_box(manifest_path),
                    black_box(target_directory),
                    black_box(binary),
                );
                argument_bytes(&metadata) + argument_bytes(&build)
            })
            .sum();
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        candidate.0 = (0..construction_count)
            .map(|_| {
                let metadata =
                    super::metadata_arguments(black_box(request), black_box(manifest_path));
                let build = super::build_arguments(
                    black_box(request),
                    black_box(manifest_path),
                    black_box(target_directory),
                    black_box(binary),
                );
                argument_bytes(&metadata) + argument_bytes(&build)
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

fn argument_bytes<T: AsRef<OsStr>>(arguments: &[T]) -> usize {
    arguments
        .iter()
        .map(|argument| argument.as_ref().len())
        .sum()
}

fn argument_text<T: AsRef<OsStr>>(arguments: &[T]) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| argument.as_ref().to_string_lossy().into_owned())
        .collect()
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
