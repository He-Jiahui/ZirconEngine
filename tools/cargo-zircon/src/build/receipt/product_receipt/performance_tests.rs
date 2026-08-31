use std::hint::black_box;
use std::time::{Duration, Instant};

use super::ProductReceipt;
use crate::build::receipt::canonical::{
    bytes_to_hex, decode_hex, decode_hex_into, INLINE_SIGNATURE_CAPACITY,
};
use crate::build::receipt::{
    validation::{
        normalize_and_validate_after_batch_shape, normalize_and_validate_owned_for_benchmark,
    },
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceiptDraft, ProductReceiptSigner,
    ProductReceiptVerifier, ReceiptArtifact, TargetProfile, ToolchainSet,
    VerifiedProductReceiptDraftHandoff,
};

const ARTIFACT_COUNT: usize = 2_048;
const FEATURE_COUNT: usize = 128;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 80;
const SINGLE_PASS_REQUIRED_PERCENT: u128 = 90;
const SINGLE_RAW_HANDOFF_REQUIRED_PERCENT: u128 = 70;
const FRESH_PUBLICATION_REQUIRED_PERCENT: u128 = 70;
const NORMALIZED_DRAFT_REQUIRED_PERCENT: u128 = 90;
const INLINE_SIGNATURE_DECODE_ITERATIONS: usize = 16_384;
const INLINE_SIGNATURE_DECODE_REQUIRED_PERCENT: u128 = 90;
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
fn borrowed_receipt_integrity_performance_evidence() {
    let receipt = benchmark_receipt();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_verification(&receipt, round % 2 == 0);
        black_box((baseline, candidate));
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_verification(&receipt, round % 2 == 0);
        baseline_samples.push(baseline);
        candidate_samples.push(candidate);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_BORROWED_RECEIPT_INTEGRITY_BENCH_V1 artifacts={ARTIFACT_COUNT} features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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
#[ignore = "release-only performance evidence"]
fn single_pass_receipt_shape_performance_evidence() {
    let receipt = benchmark_receipt();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_shape_walks(&receipt, round % 2 == 0);
        black_box((baseline, candidate));
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_shape_walks(&receipt, round % 2 == 0);
        baseline_samples.push(baseline);
        candidate_samples.push(candidate);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_SINGLE_PASS_RECEIPT_SHAPE_BENCH_V1 artifacts={ARTIFACT_COUNT} features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * SINGLE_PASS_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * SINGLE_PASS_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn raw_draft_handoff_verification_performance_evidence() {
    let draft = benchmark_draft();
    let serialized = serde_json::to_vec(&draft).unwrap();
    let expected = draft.handoff_sha256().unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_draft_handoff_verification(&serialized, &expected, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_draft_handoff_verification(&serialized, &expected, round % 2 == 0);
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
        "TOOLING15_SINGLE_RAW_DRAFT_HANDOFF_VERIFICATION_BENCH_V1 artifacts={ARTIFACT_COUNT} features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * SINGLE_RAW_HANDOFF_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * SINGLE_RAW_HANDOFF_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn fresh_verified_receipt_publication_performance_evidence() {
    let signer = BenchmarkSigner;
    let verifier = BenchmarkVerifier;
    let draft = benchmark_draft();
    let handoff_sha256 = draft.handoff_sha256().unwrap();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_publication(&draft, &handoff_sha256, &signer, &verifier, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_publication(&draft, &handoff_sha256, &signer, &verifier, round % 2 == 0);
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
        "TOOLING15_FRESH_VERIFIED_RECEIPT_PUBLICATION_BENCH_V1 artifacts={ARTIFACT_COUNT} features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * FRESH_PUBLICATION_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * FRESH_PUBLICATION_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn normalized_draft_issue_performance_evidence() {
    let draft = benchmark_draft();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_draft_normalization(&draft, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_draft_normalization(&draft, round % 2 == 0);
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
        "TOOLING15_NORMALIZED_DRAFT_ISSUE_BENCH_V1 artifacts={ARTIFACT_COUNT} features={FEATURE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * NORMALIZED_DRAFT_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * NORMALIZED_DRAFT_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn inline_signature_decode_performance_evidence() {
    let signature_hex = bytes_to_hex(&(0_u8..=63).collect::<Vec<_>>());

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_signature_decode(&signature_hex, round % 2 == 0);
        black_box((baseline, candidate));
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_signature_decode(&signature_hex, round % 2 == 0);
        baseline_samples.push(baseline);
        candidate_samples.push(candidate);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_INLINE_SIGNATURE_DECODE_BENCH_V1 signature_bytes={INLINE_SIGNATURE_CAPACITY} iterations={INLINE_SIGNATURE_DECODE_ITERATIONS} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * INLINE_SIGNATURE_DECODE_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * INLINE_SIGNATURE_DECODE_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_signature_decode(value: &str, baseline_first: bool) -> (Duration, Duration) {
    if baseline_first {
        let baseline = measure_legacy_signature_decode(value);
        let candidate = measure_inline_signature_decode(value);
        (baseline, candidate)
    } else {
        let candidate = measure_inline_signature_decode(value);
        let baseline = measure_legacy_signature_decode(value);
        (baseline, candidate)
    }
}

fn measure_legacy_signature_decode(value: &str) -> Duration {
    let started = Instant::now();
    let mut total = 0_usize;
    for _ in 0..INLINE_SIGNATURE_DECODE_ITERATIONS {
        total = total.saturating_add(decode_hex(black_box(value)).unwrap().len());
    }
    black_box(total);
    started.elapsed()
}

fn measure_inline_signature_decode(value: &str) -> Duration {
    let started = Instant::now();
    let mut inline_signature = [0_u8; INLINE_SIGNATURE_CAPACITY];
    let mut total = 0_usize;
    for _ in 0..INLINE_SIGNATURE_DECODE_ITERATIONS {
        let signature_len = decode_hex_into(black_box(value), &mut inline_signature)
            .unwrap()
            .expect("benchmark signature fits inline buffer");
        total = total.saturating_add(signature_len);
        black_box(&inline_signature[..signature_len]);
    }
    black_box(total);
    started.elapsed()
}

fn measure_draft_normalization(
    draft: &ProductReceiptDraft,
    baseline_first: bool,
) -> (
    (ProductReceiptDraft, Duration),
    (ProductReceiptDraft, Duration),
) {
    let mut baseline = draft.clone();
    let mut candidate = draft.clone();
    if baseline_first {
        let baseline = measure_owned_draft_normalization(&mut baseline);
        let candidate = measure_borrowed_draft_validation(&mut candidate);
        (baseline, candidate)
    } else {
        let candidate = measure_borrowed_draft_validation(&mut candidate);
        let baseline = measure_owned_draft_normalization(&mut baseline);
        (baseline, candidate)
    }
}

fn measure_owned_draft_normalization(
    draft: &mut ProductReceiptDraft,
) -> (ProductReceiptDraft, Duration) {
    let started = Instant::now();
    normalize_and_validate_owned_for_benchmark(black_box(draft), CREATED_UTC).unwrap();
    let elapsed = started.elapsed();
    (black_box(draft.clone()), elapsed)
}

fn measure_borrowed_draft_validation(
    draft: &mut ProductReceiptDraft,
) -> (ProductReceiptDraft, Duration) {
    let started = Instant::now();
    normalize_and_validate_after_batch_shape(black_box(draft), CREATED_UTC).unwrap();
    let elapsed = started.elapsed();
    (black_box(draft.clone()), elapsed)
}

fn measure_publication(
    draft: &ProductReceiptDraft,
    handoff_sha256: &str,
    signer: &BenchmarkSigner,
    verifier: &BenchmarkVerifier,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    let baseline = draft
        .clone()
        .verify_handoff_sha256_owned(handoff_sha256)
        .unwrap();
    let candidate = draft
        .clone()
        .verify_handoff_sha256_owned(handoff_sha256)
        .unwrap();
    if baseline_first {
        (
            measure_verified_then_public_validation(baseline, signer, verifier),
            measure_fresh_verified_publication(candidate, signer, verifier),
        )
    } else {
        let candidate = measure_fresh_verified_publication(candidate, signer, verifier);
        let baseline = measure_verified_then_public_validation(baseline, signer, verifier);
        (baseline, candidate)
    }
}

fn measure_draft_handoff_verification(
    serialized: &[u8],
    expected: &str,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let draft: ProductReceiptDraft = serde_json::from_slice(black_box(serialized)).unwrap();
        draft
            .verify_handoff_sha256_owned(black_box(expected))
            .unwrap();
        (1, started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        ProductReceiptDraft::parse_and_verify_handoff_sha256(
            black_box(serialized),
            black_box(expected),
        )
        .unwrap();
        (1, started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_verified_then_public_validation(
    draft: VerifiedProductReceiptDraftHandoff,
    signer: &BenchmarkSigner,
    verifier: &BenchmarkVerifier,
) -> (String, Duration) {
    let started = Instant::now();
    let receipt = draft.issue(CREATED_UTC, signer).unwrap();
    receipt.verify_attestation(verifier).unwrap();
    (black_box(receipt.receipt_id.clone()), started.elapsed())
}

fn measure_fresh_verified_publication(
    draft: VerifiedProductReceiptDraftHandoff,
    signer: &BenchmarkSigner,
    verifier: &BenchmarkVerifier,
) -> (String, Duration) {
    let started = Instant::now();
    let publication = draft.issue_verified(CREATED_UTC, signer, verifier).unwrap();
    (
        black_box(publication.receipt_id().to_string()),
        started.elapsed(),
    )
}

fn measure_shape_walks(receipt: &ProductReceipt, baseline_first: bool) -> (Duration, Duration) {
    if baseline_first {
        let baseline = measure_normalized_preflight(receipt);
        let candidate = measure_borrowed_verification(receipt);
        (baseline, candidate)
    } else {
        let candidate = measure_borrowed_verification(receipt);
        let baseline = measure_normalized_preflight(receipt);
        (baseline, candidate)
    }
}

fn measure_normalized_preflight(receipt: &ProductReceipt) -> Duration {
    let started = Instant::now();
    black_box(receipt)
        .verify_integrity_with_normalized_preflight()
        .unwrap();
    started.elapsed()
}

fn measure_verification(receipt: &ProductReceipt, baseline_first: bool) -> (Duration, Duration) {
    if baseline_first {
        let baseline = measure_owned_normalization(receipt);
        let candidate = measure_borrowed_verification(receipt);
        (baseline, candidate)
    } else {
        let candidate = measure_borrowed_verification(receipt);
        let baseline = measure_owned_normalization(receipt);
        (baseline, candidate)
    }
}

fn measure_owned_normalization(receipt: &ProductReceipt) -> Duration {
    let started = Instant::now();
    black_box(receipt)
        .verify_integrity_with_owned_normalization()
        .unwrap();
    started.elapsed()
}

fn measure_borrowed_verification(receipt: &ProductReceipt) -> Duration {
    let started = Instant::now();
    black_box(receipt).verify_integrity().unwrap();
    started.elapsed()
}

fn benchmark_receipt() -> ProductReceipt {
    ProductReceipt::issue(benchmark_draft(), CREATED_UTC, &BenchmarkSigner).unwrap()
}

fn benchmark_draft() -> ProductReceiptDraft {
    ProductReceiptDraft {
        build_set_id: digest('A'),
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
            package: "zircon-product".to_string(),
            bin: Some("zircon_product".to_string()),
            features: (0..FEATURE_COUNT)
                .map(|index| format!("product-feature-{index:03}"))
                .collect(),
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "0.1.0".to_string(),
            worker_id: "benchmark-worker".to_string(),
            operation_id: "benchmark-operation".to_string(),
        },
        build_products: (0..ARTIFACT_COUNT).map(benchmark_artifact).collect(),
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    }
}

fn benchmark_artifact(index: usize) -> ReceiptArtifact {
    ReceiptArtifact {
        logical_name: format!("product-artifact-{index:05}"),
        relative_path: format!("runtime/shard_{:04}/artifact_{index:05}.exe", index % 512),
        kind: ArtifactKind::Executable,
        sha256: digest(char::from_digit((index % 10) as u32, 16).unwrap()),
        byte_length: 4_096 + index as u64,
    }
}

fn digest(character: char) -> String {
    std::iter::repeat(character).take(64).collect()
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
