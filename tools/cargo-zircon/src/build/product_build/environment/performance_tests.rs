use std::hint::black_box;
use std::time::{Duration, Instant};

#[test]
#[ignore = "release-only performance evidence"]
fn single_allocation_linker_key_performance_evidence() {
    const CONSTRUCTION_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 85;

    let target_triples = (0..CONSTRUCTION_COUNT)
        .map(|index| format!("x86_64-zircon-component_{index:05}-windows-msvc"))
        .collect::<Vec<_>>();
    for target_triple in &target_triples {
        assert_eq!(
            legacy_cargo_linker_environment_key(target_triple),
            super::cargo_linker_environment_key(target_triple).unwrap()
        );
    }

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_linker_keys(&target_triples, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_linker_keys(&target_triples, round % 2 == 0);
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
        "TOOLING15_SINGLE_ALLOCATION_LINKER_KEY_BENCH_V1 constructions={CONSTRUCTION_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
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

fn measure_linker_keys(
    target_triples: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let length: usize = target_triples
            .iter()
            .map(|target| legacy_cargo_linker_environment_key(black_box(target)).len())
            .sum();
        (black_box(length), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let length: usize = target_triples
            .iter()
            .map(|target| {
                super::cargo_linker_environment_key(black_box(target))
                    .unwrap()
                    .len()
            })
            .sum();
        (black_box(length), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn legacy_cargo_linker_environment_key(target_triple: &str) -> String {
    format!(
        "CARGO_TARGET_{}_LINKER",
        target_triple
            .bytes()
            .map(|byte| match byte {
                b'a'..=b'z' => (byte - b'a' + b'A') as char,
                b'A'..=b'Z' | b'0'..=b'9' | b'_' => byte as char,
                b'-' => '_',
                _ => '?',
            })
            .collect::<String>()
    )
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
