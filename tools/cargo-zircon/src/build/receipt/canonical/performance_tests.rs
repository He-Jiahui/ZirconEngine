use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{
    attestation_bytes, batch_attestation_bytes, bytes_to_hex, canonical_build_action_key,
    canonical_build_action_sha256, sha256_serialized, upper_hex_matches, BuildAction,
    CanonicalAttestation, CanonicalBatchAttestation, PRODUCT_RECEIPT_ATTESTATION_KIND,
    PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND, PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
    PRODUCT_RECEIPT_SCHEMA_VERSION,
};

const FEATURE_COUNT: usize = 1_024;
const HASHES_PER_SAMPLE: usize = 32;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 85;
const DEDUP_ACTION_COUNT: usize = 16;
const DEDUP_FEATURE_COUNT: usize = 128;
const DEDUP_OPERATIONS_PER_SAMPLE: usize = 32;
const DEDUP_REQUIRED_PERCENT: u128 = 70;
const DIRECT_DIGEST_COUNT: usize = 16_384;
const DIRECT_DIGEST_REQUIRED_PERCENT: u128 = 70;
const ATTESTATION_PAYLOAD_SETS_PER_SAMPLE: usize = 8_192;
const ATTESTATION_PAYLOAD_REQUIRED_PERCENT: u128 = 90;

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_build_action_digest_performance_evidence() {
    let action = benchmark_action();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_digests(&action, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_digests(&action, round % 2 == 0);
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
        "TOOLING15_BORROWED_BUILD_ACTION_DIGEST_BENCH_V1 features={FEATURE_COUNT} hashes_per_sample={HASHES_PER_SAMPLE} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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

#[test]
#[ignore = "release-only performance evidence"]
fn structural_build_action_dedup_performance_evidence() {
    let actions = benchmark_actions();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_action_dedup(&actions, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_action_dedup(&actions, round % 2 == 0);
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
        "TOOLING15_STRUCTURAL_BUILD_ACTION_DEDUP_BENCH_V1 actions={DEDUP_ACTION_COUNT} features_per_action={DEDUP_FEATURE_COUNT} operations_per_sample={DEDUP_OPERATIONS_PER_SAMPLE} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * DEDUP_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * DEDUP_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn direct_digest_match_performance_evidence() {
    let fixtures = digest_fixtures();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_digest_matches(&fixtures, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_digest_matches(&fixtures, round % 2 == 0);
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
        "TOOLING15_DIRECT_DIGEST_MATCH_BENCH_V1 digests={DIRECT_DIGEST_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * DIRECT_DIGEST_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * DIRECT_DIGEST_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn preallocated_attestation_payload_performance_evidence() {
    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_attestation_payloads(round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_attestation_payloads(round % 2 == 0);
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
        "TOOLING15_PREALLOCATED_ATTESTATION_PAYLOAD_BENCH_V1 payload_sets={ATTESTATION_PAYLOAD_SETS_PER_SAMPLE} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100
            <= baseline_p50.as_nanos() * ATTESTATION_PAYLOAD_REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100
            <= baseline_p95.as_nanos() * ATTESTATION_PAYLOAD_REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_attestation_payloads(baseline_first: bool) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_legacy_attestation_payloads();
        let candidate = measure_preallocated_attestation_payloads();
        (baseline, candidate)
    } else {
        let candidate = measure_preallocated_attestation_payloads();
        let baseline = measure_legacy_attestation_payloads();
        (baseline, candidate)
    }
}

fn measure_legacy_attestation_payloads() -> (usize, Duration) {
    let receipt_id = "A".repeat(64);
    let batch_id = "B".repeat(64);
    let signer_id = "zircon-release-product-receipt-signer";
    let algorithm = "ed25519-v1";
    let started = Instant::now();
    let mut total_bytes = 0_usize;
    for _ in 0..ATTESTATION_PAYLOAD_SETS_PER_SAMPLE {
        for receipt_id in [&receipt_id, &batch_id] {
            let payload = CanonicalAttestation {
                schema_version: PRODUCT_RECEIPT_SCHEMA_VERSION,
                attestation_kind: PRODUCT_RECEIPT_ATTESTATION_KIND,
                receipt_id: black_box(receipt_id),
                signer_id: black_box(signer_id),
                algorithm: black_box(algorithm),
            };
            total_bytes = total_bytes.saturating_add(serde_json::to_vec(&payload).unwrap().len());
        }
        let payload = CanonicalBatchAttestation {
            schema_version: PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
            attestation_kind: PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND,
            batch_id: black_box(&batch_id),
            signer_id: black_box(signer_id),
            algorithm: black_box(algorithm),
        };
        total_bytes = total_bytes.saturating_add(serde_json::to_vec(&payload).unwrap().len());
    }
    (black_box(total_bytes), started.elapsed())
}

fn measure_preallocated_attestation_payloads() -> (usize, Duration) {
    let receipt_id = "A".repeat(64);
    let batch_id = "B".repeat(64);
    let signer_id = "zircon-release-product-receipt-signer";
    let algorithm = "ed25519-v1";
    let started = Instant::now();
    let mut total_bytes = 0_usize;
    for _ in 0..ATTESTATION_PAYLOAD_SETS_PER_SAMPLE {
        total_bytes = total_bytes.saturating_add(
            attestation_bytes(black_box(&receipt_id), signer_id, algorithm)
                .unwrap()
                .len(),
        );
        total_bytes = total_bytes.saturating_add(
            attestation_bytes(black_box(&batch_id), signer_id, algorithm)
                .unwrap()
                .len(),
        );
        total_bytes = total_bytes.saturating_add(
            batch_attestation_bytes(black_box(&batch_id), signer_id, algorithm)
                .unwrap()
                .len(),
        );
    }
    (black_box(total_bytes), started.elapsed())
}

fn digest_fixtures() -> Vec<([u8; 32], String)> {
    (0..DIRECT_DIGEST_COUNT)
        .map(|index| {
            let mut digest = [0_u8; 32];
            for (offset, byte) in digest.iter_mut().enumerate() {
                *byte = index.wrapping_mul(31).wrapping_add(offset.wrapping_mul(17)) as u8;
            }
            let expected = bytes_to_hex(&digest);
            (digest, expected)
        })
        .collect()
}

fn measure_digest_matches(
    fixtures: &[([u8; 32], String)],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_encoded_digest_matches(fixtures);
        let candidate = measure_direct_digest_matches(fixtures);
        (baseline, candidate)
    } else {
        let candidate = measure_direct_digest_matches(fixtures);
        let baseline = measure_encoded_digest_matches(fixtures);
        (baseline, candidate)
    }
}

fn measure_encoded_digest_matches(fixtures: &[([u8; 32], String)]) -> (usize, Duration) {
    let started = Instant::now();
    let mut matched = 0_usize;
    for (digest, expected) in fixtures {
        matched += usize::from(bytes_to_hex(black_box(digest)) == black_box(expected.as_str()));
    }
    (black_box(matched), started.elapsed())
}

fn measure_direct_digest_matches(fixtures: &[([u8; 32], String)]) -> (usize, Duration) {
    let started = Instant::now();
    let mut matched = 0_usize;
    for (digest, expected) in fixtures {
        matched += usize::from(upper_hex_matches(
            black_box(digest),
            black_box(expected.as_str()),
        ));
    }
    (black_box(matched), started.elapsed())
}

fn benchmark_action() -> BuildAction {
    BuildAction {
        package: "zircon-editor".to_string(),
        bin: Some("zircon_editor".to_string()),
        features: (0..FEATURE_COUNT)
            .map(|index| {
                format!("feature_{index:04}_with_a_representative_product_configuration_suffix")
            })
            .collect(),
    }
}

fn benchmark_actions() -> Vec<BuildAction> {
    (0..DEDUP_ACTION_COUNT)
        .map(|action_index| BuildAction {
            package: format!("zircon-product-{action_index:02}"),
            bin: Some(format!("zircon_product_{action_index:02}")),
            features: (0..DEDUP_FEATURE_COUNT)
                .map(|feature_index| {
                    format!(
                        "action_{action_index:02}_feature_{feature_index:04}_representative_suffix"
                    )
                })
                .collect(),
        })
        .collect()
}

fn measure_digests(
    action: &BuildAction,
    baseline_first: bool,
) -> ((String, Duration), (String, Duration)) {
    if baseline_first {
        let baseline = measure_legacy(action);
        let candidate = measure_candidate(action);
        (baseline, candidate)
    } else {
        let candidate = measure_candidate(action);
        let baseline = measure_legacy(action);
        (baseline, candidate)
    }
}

fn measure_legacy(action: &BuildAction) -> (String, Duration) {
    let started = Instant::now();
    let mut digest = String::new();
    for _ in 0..HASHES_PER_SAMPLE {
        let action = black_box(action);
        let mut features = action
            .features
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        features.sort_unstable();
        let legacy = LegacyCanonicalBuildActionKey {
            package: &action.package,
            bin: action.bin.as_deref(),
            features,
        };
        digest = sha256_serialized(&legacy, "legacy build action").unwrap();
        black_box(&digest);
    }
    (digest, started.elapsed())
}

#[derive(serde::Serialize)]
struct LegacyCanonicalBuildActionKey<'a> {
    package: &'a str,
    bin: Option<&'a str>,
    features: Vec<&'a str>,
}

fn measure_candidate(action: &BuildAction) -> (String, Duration) {
    let started = Instant::now();
    let mut digest = String::new();
    for _ in 0..HASHES_PER_SAMPLE {
        digest = canonical_build_action_sha256(black_box(action)).unwrap();
        black_box(&digest);
    }
    (digest, started.elapsed())
}

fn measure_action_dedup(
    actions: &[BuildAction],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        let baseline = measure_sha_action_dedup(actions);
        let candidate = measure_structural_action_dedup(actions);
        (baseline, candidate)
    } else {
        let candidate = measure_structural_action_dedup(actions);
        let baseline = measure_sha_action_dedup(actions);
        (baseline, candidate)
    }
}

fn measure_sha_action_dedup(actions: &[BuildAction]) -> (usize, Duration) {
    let started = Instant::now();
    let mut accepted = 0_usize;
    for _ in 0..DEDUP_OPERATIONS_PER_SAMPLE {
        let mut identities = HashSet::with_capacity(actions.len());
        for action in actions {
            assert!(identities.insert(canonical_build_action_sha256(black_box(action)).unwrap()));
        }
        accepted = accepted.saturating_add(identities.len());
    }
    (accepted, started.elapsed())
}

fn measure_structural_action_dedup(actions: &[BuildAction]) -> (usize, Duration) {
    let started = Instant::now();
    let mut accepted = 0_usize;
    for _ in 0..DEDUP_OPERATIONS_PER_SAMPLE {
        let mut identities = HashSet::with_capacity(actions.len());
        for action in actions {
            assert!(identities.insert(canonical_build_action_key(black_box(action))));
        }
        accepted = accepted.saturating_add(identities.len());
    }
    (accepted, started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
