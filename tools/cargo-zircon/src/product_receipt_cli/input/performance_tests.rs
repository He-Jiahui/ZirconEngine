use std::hint::black_box;
use std::io::Cursor;
use std::time::{Duration, Instant};

use super::read_bounded_from;

const ROUTES_PER_SAMPLE: usize = 512;
const DRAFT_BYTES: usize = 16 * 1024;
const TRUST_BYTES: usize = 1024;
const KEY_BYTES: usize = 256;
const JSON_LIMIT: usize = 16 * 1024 * 1024;
const TRUST_LIMIT: usize = 1024 * 1024;
const KEY_LIMIT: usize = 16 * 1024;
const WARMUP_ROUNDS: usize = 3;
const SAMPLE_ROUNDS: usize = 51;
const REQUIRED_PERCENT: u128 = 85;

#[test]
#[ignore = "release-only performance evidence"]
fn reused_bounded_input_buffer_performance_evidence() {
    let draft = vec![b'D'; DRAFT_BYTES];
    let trust = vec![b'T'; TRUST_BYTES];
    let key = vec![b'K'; KEY_BYTES];

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_routes(&draft, &trust, &key, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_routes(&draft, &trust, &key, round % 2 == 0);
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
        "TOOLING15_REUSED_BOUNDED_INPUT_BUFFER_BENCH_V1 routes={ROUTES_PER_SAMPLE} draft_bytes={DRAFT_BYTES} trust_bytes={TRUST_BYTES} key_bytes={KEY_BYTES} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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

fn measure_routes(
    draft: &[u8],
    trust: &[u8],
    key: &[u8],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    if baseline_first {
        (
            measure_separate_buffers(draft, trust, key),
            measure_reused_buffer(draft, trust, key),
        )
    } else {
        let candidate = measure_reused_buffer(draft, trust, key);
        let baseline = measure_separate_buffers(draft, trust, key);
        (baseline, candidate)
    }
}

fn measure_separate_buffers(draft: &[u8], trust: &[u8], key: &[u8]) -> (usize, Duration) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..ROUTES_PER_SAMPLE {
        let mut draft_buffer = Vec::new();
        checksum = checksum.saturating_add(
            read_bounded_from(
                Cursor::new(black_box(draft)),
                JSON_LIMIT,
                "draft",
                &mut draft_buffer,
            )
            .unwrap()
            .len(),
        );
        let mut trust_buffer = Vec::new();
        checksum = checksum.saturating_add(
            read_bounded_from(
                Cursor::new(black_box(trust)),
                TRUST_LIMIT,
                "trust registry",
                &mut trust_buffer,
            )
            .unwrap()
            .len(),
        );
        let mut key_buffer = Vec::new();
        checksum = checksum.saturating_add(
            read_bounded_from(
                Cursor::new(black_box(key)),
                KEY_LIMIT,
                "private key",
                &mut key_buffer,
            )
            .unwrap()
            .len(),
        );
    }
    (black_box(checksum), started.elapsed())
}

fn measure_reused_buffer(draft: &[u8], trust: &[u8], key: &[u8]) -> (usize, Duration) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..ROUTES_PER_SAMPLE {
        let mut input = Vec::new();
        checksum = checksum.saturating_add(
            read_bounded_from(
                Cursor::new(black_box(draft)),
                JSON_LIMIT,
                "draft",
                &mut input,
            )
            .unwrap()
            .len(),
        );
        checksum = checksum.saturating_add(
            read_bounded_from(
                Cursor::new(black_box(trust)),
                TRUST_LIMIT,
                "trust registry",
                &mut input,
            )
            .unwrap()
            .len(),
        );
        checksum = checksum.saturating_add(
            read_bounded_from(
                Cursor::new(black_box(key)),
                KEY_LIMIT,
                "private key",
                &mut input,
            )
            .unwrap()
            .len(),
        );
    }
    (black_box(checksum), started.elapsed())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
