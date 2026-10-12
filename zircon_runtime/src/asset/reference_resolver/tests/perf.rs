use std::hint::black_box;
use std::time::Instant;

const ROOT_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn legacy_candidates(matches: &[bool]) -> usize {
    let mut candidates = Vec::new();
    for (index, matches) in matches.iter().copied().enumerate() {
        if matches {
            candidates.push(index);
        }
    }
    candidates.len()
}

fn optimized_candidates(matches: &[bool]) -> usize {
    let mut candidates = Vec::with_capacity(matches.len());
    for (index, matches) in matches.iter().copied().enumerate() {
        if matches {
            candidates.push(index);
        }
    }
    candidates.len()
}

#[test]
fn optimization_batch_r6_wave_runtime631_reference_candidates_reserve_root_bounds() {
    let source = include_str!("../../reference_resolver.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("reference resolver production source");

    assert_eq!(
        production
            .matches("let mut candidates = Vec::with_capacity(roots.len());")
            .count(),
        2
    );
    assert!(!production.contains("let mut candidates = Vec::new();"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_r6_wave_runtime631_reference_candidate_capacity_performance_evidence() {
    let matches = vec![true; ROOT_COUNT];
    assert_eq!(legacy_candidates(&matches), optimized_candidates(&matches));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_candidates(black_box(&matches)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_candidates(black_box(&matches)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_candidates(black_box(&matches)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_candidates(black_box(&matches)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "RUNTIME631_PREALLOCATED_REFERENCE_CANDIDATE_BENCH_V1 roots={ROOT_COUNT} \
         selected={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=8500",
        optimized_candidates(&matches)
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated reference candidates P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
