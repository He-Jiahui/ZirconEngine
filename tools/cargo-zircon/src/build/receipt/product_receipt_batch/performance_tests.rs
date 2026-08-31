use std::hint::black_box;
use std::time::{Duration, Instant};

use super::ProductReceiptBatch;
use crate::build::receipt::{
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceipt, ProductReceiptDraft,
    ProductReceiptSigner, ReceiptArtifact, TargetProfile, ToolchainSet,
};

const RECEIPT_COUNT: usize = 16;
const ARTIFACTS_PER_RECEIPT: usize = 128;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 70;
const FRESH_RECEIPT_REQUIRED_PERCENT: u128 = 50;
const VALIDATED_CLOSURE_REQUIRED_PERCENT: u128 = 90;
const CREATED_UTC: &str = "2026-08-29T00:00:00Z";

struct BenchmarkSigner;

impl ProductReceiptSigner for BenchmarkSigner {
    fn signer_id(&self) -> &str {
        "tooling15-benchmark-signer"
    }

    fn algorithm(&self) -> &str {
        "benchmark-signature-v1"
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![payload.len() as u8; 64])
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn batch_issue_single_validation_performance_evidence() {
    let signer = BenchmarkSigner;
    let build_set_id = digest('A');
    let receipts = (0..RECEIPT_COUNT)
        .map(|receipt_index| benchmark_receipt(receipt_index, &build_set_id, &signer))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_issue(&build_set_id, &receipts, &signer, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_issue(&build_set_id, &receipts, &signer, round % 2 == 0);
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
        "TOOLING15_BATCH_ISSUE_SINGLE_VALIDATION_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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
fn fresh_receipt_batch_issue_performance_evidence() {
    let signer = BenchmarkSigner;
    let build_set_id = digest('A');
    let receipts = (0..RECEIPT_COUNT)
        .map(|receipt_index| benchmark_receipt(receipt_index, &build_set_id, &signer))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_fresh_issue(&build_set_id, &receipts, &signer, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_fresh_issue(&build_set_id, &receipts, &signer, round % 2 == 0);
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
        "TOOLING15_FRESH_RECEIPT_BATCH_ISSUE_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * FRESH_RECEIPT_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 50%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * FRESH_RECEIPT_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 50%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn validated_batch_closure_performance_evidence() {
    let signer = BenchmarkSigner;
    let build_set_id = digest('A');
    let receipts = (0..RECEIPT_COUNT)
        .map(|receipt_index| benchmark_receipt(receipt_index, &build_set_id, &signer))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_validated_closure(&build_set_id, &receipts, &signer, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_validated_closure(&build_set_id, &receipts, &signer, round % 2 == 0);
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
        "TOOLING15_VALIDATED_BATCH_CLOSURE_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * VALIDATED_CLOSURE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * VALIDATED_CLOSURE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_validated_closure(
    build_set_id: &str,
    receipts: &[ProductReceipt],
    signer: &BenchmarkSigner,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let baseline_receipts = receipts.to_vec();
    let candidate_receipts = receipts.to_vec();
    if baseline_first {
        (
            measure_shape_validating_issue(build_set_id, baseline_receipts, signer),
            measure_validated_closure_issue(build_set_id, candidate_receipts, signer),
        )
    } else {
        let candidate = measure_validated_closure_issue(build_set_id, candidate_receipts, signer);
        let baseline = measure_shape_validating_issue(build_set_id, baseline_receipts, signer);
        (baseline, candidate)
    }
}

fn measure_shape_validating_issue(
    build_set_id: &str,
    receipts: Vec<ProductReceipt>,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let batch = ProductReceiptBatch::issue_after_receipt_integrity_with_shape_validation(
        build_set_id.to_string(),
        receipts,
        signer,
    )
    .unwrap();
    (black_box(batch.batch_id), started.elapsed())
}

fn measure_validated_closure_issue(
    build_set_id: &str,
    receipts: Vec<ProductReceipt>,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let batch = ProductReceiptBatch::issue_after_validated_closure(
        build_set_id.to_string(),
        receipts,
        signer,
    )
    .unwrap();
    (black_box(batch.batch_id), started.elapsed())
}

fn measure_fresh_issue(
    build_set_id: &str,
    receipts: &[ProductReceipt],
    signer: &BenchmarkSigner,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let baseline_receipts = receipts.to_vec();
    let candidate_receipts = receipts.to_vec();
    if baseline_first {
        (
            measure_public_issue(build_set_id, baseline_receipts, signer),
            measure_fresh_receipt_issue(build_set_id, candidate_receipts, signer),
        )
    } else {
        let candidate = measure_fresh_receipt_issue(build_set_id, candidate_receipts, signer);
        let baseline = measure_public_issue(build_set_id, baseline_receipts, signer);
        (baseline, candidate)
    }
}

fn measure_public_issue(
    build_set_id: &str,
    receipts: Vec<ProductReceipt>,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let batch = ProductReceiptBatch::issue(build_set_id.to_string(), receipts, signer).unwrap();
    (black_box(batch.batch_id), started.elapsed())
}

fn measure_fresh_receipt_issue(
    build_set_id: &str,
    receipts: Vec<ProductReceipt>,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let batch = ProductReceiptBatch::issue_after_validated_closure(
        build_set_id.to_string(),
        receipts,
        signer,
    )
    .unwrap();
    (black_box(batch.batch_id), started.elapsed())
}

fn measure_issue(
    build_set_id: &str,
    receipts: &[ProductReceipt],
    signer: &BenchmarkSigner,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let baseline_receipts = receipts.to_vec();
    let candidate_receipts = receipts.to_vec();
    if baseline_first {
        (
            measure_legacy_issue(build_set_id, baseline_receipts, signer),
            measure_candidate_issue(build_set_id, candidate_receipts, signer),
        )
    } else {
        let candidate = measure_candidate_issue(build_set_id, candidate_receipts, signer);
        let baseline = measure_legacy_issue(build_set_id, baseline_receipts, signer);
        (baseline, candidate)
    }
}

fn measure_legacy_issue(
    build_set_id: &str,
    receipts: Vec<ProductReceipt>,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let batch = ProductReceiptBatch::issue(build_set_id.to_string(), receipts, signer).unwrap();
    batch.verify_integrity().unwrap();
    (black_box(batch.batch_id), started.elapsed())
}

fn measure_candidate_issue(
    build_set_id: &str,
    receipts: Vec<ProductReceipt>,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let batch = ProductReceiptBatch::issue(build_set_id.to_string(), receipts, signer).unwrap();
    (black_box(batch.batch_id), started.elapsed())
}

fn benchmark_receipt(
    receipt_index: usize,
    build_set_id: &str,
    signer: &BenchmarkSigner,
) -> ProductReceipt {
    let artifacts = (0..ARTIFACTS_PER_RECEIPT)
        .map(|artifact_index| ReceiptArtifact {
            logical_name: format!("receipt-{receipt_index}-artifact-{artifact_index}"),
            relative_path: format!("receipt-{receipt_index}/artifact-{artifact_index:04}.exe"),
            kind: ArtifactKind::Executable,
            sha256: digest('B'),
            byte_length: 1_024,
        })
        .collect();
    let draft = ProductReceiptDraft {
        build_set_id: build_set_id.to_string(),
        toolchain: ToolchainSet::new(
            digest('C'),
            digest('D'),
            Some(digest('E')),
            digest('F'),
            digest('0'),
        )
        .unwrap(),
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            codegen_flags_digest: digest('1'),
            cargo_graph_digest: digest('2'),
        },
        action: BuildAction {
            package: format!("product-{receipt_index}"),
            bin: Some(format!("product-{receipt_index}")),
            features: vec![format!("feature-{receipt_index}")],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "benchmark".to_string(),
            worker_id: "benchmark-worker".to_string(),
            operation_id: format!("operation-{receipt_index}"),
        },
        build_products: artifacts,
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    };
    ProductReceipt::issue(draft, CREATED_UTC, signer).unwrap()
}

fn digest(character: char) -> String {
    character.to_string().repeat(64)
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
