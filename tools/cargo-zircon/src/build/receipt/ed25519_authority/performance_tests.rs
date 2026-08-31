use std::collections::{hash_map::Entry, HashMap};
use std::hint::black_box;
use std::time::{Duration, Instant};

const SIGNER_COUNT: usize = 4_096;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 80;

#[test]
#[ignore = "release-only performance evidence"]
fn trust_registry_key_move_performance_evidence() {
    let signer_ids = (0..SIGNER_COUNT)
        .map(|index| format!("build-worker-{index:06}-{}", "x".repeat(96)))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_insertions(&signer_ids, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_insertions(&signer_ids, round % 2 == 0);
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
        "TOOLING15_TRUST_REGISTRY_KEY_MOVE_BENCH_V1 signers={SIGNER_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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

fn measure_insertions(
    signer_ids: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let baseline_ids = signer_ids.to_vec();
    let candidate_ids = signer_ids.to_vec();
    if baseline_first {
        let baseline = measure_cloned_keys(baseline_ids);
        let candidate = measure_moved_keys(candidate_ids);
        (baseline, candidate)
    } else {
        let candidate = measure_moved_keys(candidate_ids);
        let baseline = measure_cloned_keys(baseline_ids);
        (baseline, candidate)
    }
}

fn measure_cloned_keys(signer_ids: Vec<String>) -> (usize, Duration) {
    let started = Instant::now();
    let mut issuers = HashMap::with_capacity(signer_ids.len());
    for signer_id in signer_ids {
        assert!(issuers.insert(signer_id.clone(), ()).is_none());
        black_box(signer_id);
    }
    (black_box(issuers.len()), started.elapsed())
}

fn measure_moved_keys(signer_ids: Vec<String>) -> (usize, Duration) {
    let started = Instant::now();
    let mut issuers = HashMap::with_capacity(signer_ids.len());
    for signer_id in signer_ids {
        let Entry::Vacant(entry) = issuers.entry(signer_id) else {
            panic!("benchmark signer IDs must be unique");
        };
        entry.insert(());
    }
    (black_box(issuers.len()), started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
