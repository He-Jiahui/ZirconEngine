use std::hint::black_box;
use std::io;
use std::time::{Duration, Instant};

use crate::build::receipt::{
    receipt_writer, validate_created_utc_for_batch, ArtifactKind, BuildAction, ProducerIdentity,
    ProductReceiptDraft, ProductReceiptSigner, ProductReceiptVerifier, ReceiptArtifact,
    TargetProfile, ToolchainSet,
};

use super::ProductBuildDraftBatch;

const RECEIPT_COUNT: usize = 16;
const ARTIFACTS_PER_RECEIPT: usize = 128;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 90;
const VERIFIED_PUBLICATION_REQUIRED_PERCENT: u128 = 70;
const TIMESTAMP_VALIDATION_OPERATIONS: usize = 4_096;
const TIMESTAMP_REQUIRED_PERCENT: u128 = 30;
const DRAFT_BATCH_WRITE_REQUIRED_PERCENT: u128 = 65;
const RAW_HANDOFF_REQUIRED_PERCENT: u128 = 70;
const CREATED_UTC: &str = "2026-08-29T00:00:00Z";

struct BenchmarkSigner;

impl ProductReceiptSigner for BenchmarkSigner {
    fn signer_id(&self) -> &str {
        "benchmark-worker"
    }

    fn algorithm(&self) -> &str {
        "benchmark-signature-v1"
    }

    fn sign(&self, _attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![0xA5; 64])
    }
}

struct BenchmarkVerifier;

impl ProductReceiptVerifier for BenchmarkVerifier {
    fn verify(
        &self,
        _signer_id: &str,
        _algorithm: &str,
        _attestation_payload: &[u8],
        _signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_validation_draft_batch_write_performance_evidence() {
    let batch = benchmark_batch();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_draft_batch_write_validation(&batch, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_draft_batch_write_validation(&batch, round % 2 == 0);
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
        "TOOLING15_SINGLE_VALIDATION_DRAFT_BATCH_WRITE_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * DRAFT_BATCH_WRITE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 35%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * DRAFT_BATCH_WRITE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 35%"
    );
}

fn measure_draft_batch_write_validation(
    batch: &ProductBuildDraftBatch,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let digest = black_box(batch).handoff_sha256().unwrap();
        black_box(batch).validate_shape().unwrap();
        black_box(serde_json::to_vec_pretty(black_box(batch)).unwrap());
        (digest, started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        black_box(batch).validate_shape().unwrap();
        let digest =
            receipt_writer::write_canonical_json_with_sha256(io::sink(), black_box(batch)).unwrap();
        (digest, started.elapsed())
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
fn raw_draft_handoff_verification_performance_evidence() {
    let batch = benchmark_batch();
    let serialized = serde_json::to_vec(&batch).unwrap();
    let expected = batch.handoff_sha256().unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_handoff_verification(&serialized, &expected, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_handoff_verification(&serialized, &expected, round % 2 == 0);
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
        "TOOLING15_RAW_DRAFT_HANDOFF_VERIFICATION_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * RAW_HANDOFF_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * RAW_HANDOFF_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

fn measure_handoff_verification(
    serialized: &[u8],
    expected: &str,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let batch: ProductBuildDraftBatch = serde_json::from_slice(black_box(serialized)).unwrap();
        batch
            .verify_handoff_sha256_owned(black_box(expected))
            .unwrap();
        (RECEIPT_COUNT, started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        ProductBuildDraftBatch::parse_and_verify_handoff_sha256(
            black_box(serialized),
            black_box(expected),
        )
        .unwrap();
        (RECEIPT_COUNT, started.elapsed())
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
fn verified_draft_batch_issue_performance_evidence() {
    let signer = BenchmarkSigner;
    let batch = benchmark_batch();
    let handoff_sha256 = batch.handoff_sha256().unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_issue(&batch, &handoff_sha256, &signer, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_issue(&batch, &handoff_sha256, &signer, round % 2 == 0);
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
        "TOOLING15_VERIFIED_DRAFT_BATCH_ISSUE_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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

#[test]
#[ignore = "release-only performance evidence"]
fn fresh_verified_publication_performance_evidence() {
    let signer = BenchmarkSigner;
    let verifier = BenchmarkVerifier;
    let batch = benchmark_batch();
    let handoff_sha256 = batch.handoff_sha256().unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_publication(&batch, &handoff_sha256, &signer, &verifier, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_publication(&batch, &handoff_sha256, &signer, &verifier, round % 2 == 0);
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
        "TOOLING15_FRESH_VERIFIED_PUBLICATION_BENCH_V1 receipts={RECEIPT_COUNT} artifacts_per_receipt={ARTIFACTS_PER_RECEIPT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * VERIFIED_PUBLICATION_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * VERIFIED_PUBLICATION_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn batch_timestamp_validation_performance_evidence() {
    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_timestamp_validation(round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_timestamp_validation(round % 2 == 0);
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
        "TOOLING15_BATCH_TIMESTAMP_VALIDATION_BENCH_V1 receipts={RECEIPT_COUNT} operations={TIMESTAMP_VALIDATION_OPERATIONS} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * TIMESTAMP_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 70%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * TIMESTAMP_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 70%"
    );
}

fn measure_timestamp_validation(baseline_first: bool) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let mut accepted = 0_usize;
        for _ in 0..TIMESTAMP_VALIDATION_OPERATIONS {
            for _ in 0..RECEIPT_COUNT {
                black_box(validate_created_utc_for_batch(black_box(CREATED_UTC)).unwrap());
                accepted += 1;
            }
        }
        (accepted, started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let mut accepted = 0_usize;
        for _ in 0..TIMESTAMP_VALIDATION_OPERATIONS {
            black_box(validate_created_utc_for_batch(black_box(CREATED_UTC)).unwrap());
            accepted += RECEIPT_COUNT;
        }
        (accepted, started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_publication(
    batch: &ProductBuildDraftBatch,
    handoff_sha256: &str,
    signer: &BenchmarkSigner,
    verifier: &BenchmarkVerifier,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let baseline_batch = batch
        .clone()
        .verify_handoff_sha256_owned(handoff_sha256)
        .unwrap();
    let candidate_batch = batch
        .clone()
        .verify_handoff_sha256_owned(handoff_sha256)
        .unwrap();
    if baseline_first {
        let baseline = measure_verified_then_public_validation(baseline_batch, signer, verifier);
        let candidate = measure_fresh_verified_publication(candidate_batch, signer, verifier);
        (baseline, candidate)
    } else {
        let candidate = measure_fresh_verified_publication(candidate_batch, signer, verifier);
        let baseline = measure_verified_then_public_validation(baseline_batch, signer, verifier);
        (baseline, candidate)
    }
}

fn measure_verified_then_public_validation(
    batch: super::VerifiedProductBuildDraftBatchHandoff,
    signer: &BenchmarkSigner,
    verifier: &BenchmarkVerifier,
) -> (String, Duration) {
    let started = Instant::now();
    let receipt = batch.issue(CREATED_UTC, signer).unwrap();
    receipt.verify_attestations(verifier).unwrap();
    (black_box(receipt.batch_id.clone()), started.elapsed())
}

fn measure_fresh_verified_publication(
    batch: super::VerifiedProductBuildDraftBatchHandoff,
    signer: &BenchmarkSigner,
    verifier: &BenchmarkVerifier,
) -> (String, Duration) {
    let started = Instant::now();
    let publication = batch.issue_verified(CREATED_UTC, signer, verifier).unwrap();
    (
        black_box(publication.batch_id().to_string()),
        started.elapsed(),
    )
}

fn measure_issue(
    batch: &ProductBuildDraftBatch,
    handoff_sha256: &str,
    signer: &BenchmarkSigner,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let baseline_batch = batch.clone();
    let candidate_batch = batch
        .clone()
        .verify_handoff_sha256_owned(handoff_sha256)
        .unwrap();
    if baseline_first {
        let baseline = measure_public_issue(baseline_batch, signer);
        let candidate = measure_verified_issue(candidate_batch, signer);
        (baseline, candidate)
    } else {
        let candidate = measure_verified_issue(candidate_batch, signer);
        let baseline = measure_public_issue(baseline_batch, signer);
        (baseline, candidate)
    }
}

fn measure_public_issue(
    batch: ProductBuildDraftBatch,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let receipt = batch.issue(CREATED_UTC, signer).unwrap();
    (black_box(receipt.batch_id), started.elapsed())
}

fn measure_verified_issue(
    batch: super::VerifiedProductBuildDraftBatchHandoff,
    signer: &BenchmarkSigner,
) -> (String, Duration) {
    let started = Instant::now();
    let receipt = batch.issue(CREATED_UTC, signer).unwrap();
    (black_box(receipt.batch_id), started.elapsed())
}

fn benchmark_batch() -> ProductBuildDraftBatch {
    let build_set_id = digest('A');
    ProductBuildDraftBatch {
        schema_version: 1,
        draft_batch_kind: "zircon_product_build_draft_batch".to_string(),
        build_set_id: build_set_id.clone(),
        drafts: (0..RECEIPT_COUNT)
            .map(|receipt_index| benchmark_draft(receipt_index, &build_set_id))
            .collect(),
    }
}

fn benchmark_draft(receipt_index: usize, build_set_id: &str) -> ProductReceiptDraft {
    let build_products = vec![benchmark_artifact(
        receipt_index,
        0,
        ArtifactKind::Executable,
    )];
    let runtime_dependencies = (1..ARTIFACTS_PER_RECEIPT)
        .map(|artifact_index| {
            benchmark_artifact(receipt_index, artifact_index, ArtifactKind::DynamicLibrary)
        })
        .collect();
    ProductReceiptDraft {
        build_set_id: build_set_id.to_string(),
        toolchain: ToolchainSet::new(
            digest('B'),
            digest('C'),
            Some(digest('D')),
            digest('E'),
            digest('F'),
        )
        .unwrap(),
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            codegen_flags_digest: digest('1'),
            cargo_graph_digest: digest('2'),
        },
        action: BuildAction {
            package: format!("zircon-product-{receipt_index:02}"),
            bin: Some(format!("zircon_product_{receipt_index:02}")),
            features: vec![format!("target-product-{receipt_index:02}")],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "0.1.0".to_string(),
            worker_id: "benchmark-worker".to_string(),
            operation_id: format!("benchmark-operation-{receipt_index:02}"),
        },
        build_products,
        runtime_dependencies,
        symbols: Vec::new(),
        sbom: None,
    }
}

fn benchmark_artifact(
    receipt_index: usize,
    artifact_index: usize,
    kind: ArtifactKind,
) -> ReceiptArtifact {
    ReceiptArtifact {
        logical_name: format!("artifact-{receipt_index:02}-{artifact_index:03}"),
        relative_path: format!(
            "product_{receipt_index:02}/artifact_{artifact_index:03}.{}",
            if artifact_index == 0 { "exe" } else { "dll" }
        ),
        kind,
        sha256: digest(char::from_digit((artifact_index % 10) as u32, 16).unwrap()),
        byte_length: 4_096 + artifact_index as u64,
    }
}

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
