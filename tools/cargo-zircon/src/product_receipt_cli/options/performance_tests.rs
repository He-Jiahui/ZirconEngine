use std::hint::black_box;
use std::time::{Duration, Instant};

use super::parse_product_receipt_draft_batch_issue_options;

const ROUTES_PER_SAMPLE: usize = 4_096;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 90;

#[test]
#[ignore = "release-only performance evidence"]
fn single_pass_cli_argument_route_performance_evidence() {
    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_routes(round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_routes(round % 2 == 0);
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
        "TOOLING15_SINGLE_PASS_CLI_ARGUMENT_ROUTE_BENCH_V1 routes={ROUTES_PER_SAMPLE} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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

fn measure_routes(baseline_first: bool) -> ((usize, Duration), (usize, Duration)) {
    let baseline_inputs = benchmark_arguments_batch();
    let candidate_inputs = benchmark_arguments_batch();
    if baseline_first {
        (
            measure_shifted_routes(baseline_inputs),
            measure_single_pass_routes(candidate_inputs),
        )
    } else {
        let candidate = measure_single_pass_routes(candidate_inputs);
        let baseline = measure_shifted_routes(baseline_inputs);
        (baseline, candidate)
    }
}

fn measure_shifted_routes(inputs: Vec<Vec<String>>) -> (usize, Duration) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for mut arguments in inputs {
        let command = arguments.remove(0);
        assert_eq!(command, "issue-draft-batch");
        let options = parse_product_receipt_draft_batch_issue_options(arguments).unwrap();
        checksum = checksum
            .saturating_add(options.expected_draft_sha256.len())
            .saturating_add(options.signer_id.len())
            .saturating_add(options.created_utc.len());
    }
    (black_box(checksum), started.elapsed())
}

fn measure_single_pass_routes(inputs: Vec<Vec<String>>) -> (usize, Duration) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for arguments in inputs {
        let mut arguments = arguments.into_iter();
        let command = arguments.next().unwrap();
        assert_eq!(command, "issue-draft-batch");
        let options = parse_product_receipt_draft_batch_issue_options(arguments).unwrap();
        checksum = checksum
            .saturating_add(options.expected_draft_sha256.len())
            .saturating_add(options.signer_id.len())
            .saturating_add(options.created_utc.len());
    }
    (black_box(checksum), started.elapsed())
}

fn benchmark_arguments_batch() -> Vec<Vec<String>> {
    (0..ROUTES_PER_SAMPLE)
        .map(|_| {
            [
                "issue-draft-batch",
                "--draft-batch",
                "draft-batch.json",
                "--expected-draft-sha256",
                "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789",
                "--private-key",
                "issuer.pk8",
                "--trust-registry",
                "trust-registry.json",
                "--signer-id",
                "build-worker-01",
                "--created-utc",
                "2026-08-29T00:00:00Z",
                "--output",
                "product-receipt-batch.json",
            ]
            .into_iter()
            .map(str::to_string)
            .collect()
        })
        .collect()
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
