use std::hint::black_box;
use std::time::Instant;

use super::store_unique_candidate;

const MARKER: &str = "RUNTIME239_PERSISTED_REFERENCE_UNIQUE_CANDIDATE_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 65_536;

#[test]
fn optimization_batch_20260826gs_runtime239_unique_candidate_preserves_first_and_marks_ambiguity() {
    let mut unique = None;
    assert!(!store_unique_candidate(&mut unique, "first"));
    assert_eq!(unique, Some("first"));

    assert!(store_unique_candidate(&mut unique, "second"));
    assert_eq!(unique, Some("first"));
    assert!(store_unique_candidate(&mut unique, "third"));
    assert_eq!(unique, Some("first"));
}

#[test]
fn optimization_batch_20260826gs_runtime239_persisted_reference_avoids_candidate_vec() {
    let source = include_str!("../persisted_reference.rs");
    assert!(source.contains("store_unique_candidate(&mut unique_candidate"));
    assert!(source.contains("let mut unique_candidate = None;"));
    assert!(!source.contains("let mut candidates = Vec::new();"));
    assert!(!source.contains("match candidates.as_slice()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gs_runtime239_persisted_reference_unique_candidate_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_single_candidate));
            optimized_samples.push(measure(optimized_single_candidate));
        } else {
            optimized_samples.push(measure(optimized_single_candidate));
            legacy_samples.push(measure(legacy_single_candidate));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "unique candidate storage must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_single_candidate() -> usize {
    let mut candidates = Vec::new();
    candidates.push(black_box([7_usize; 8]));
    black_box(&candidates);
    black_box(candidates[0][0])
}

fn optimized_single_candidate() -> usize {
    let mut candidate = None;
    black_box(store_unique_candidate(
        &mut candidate,
        black_box([7_usize; 8]),
    ));
    black_box(&candidate);
    black_box(candidate.expect("candidate should be stored")[0])
}

fn measure(implementation: fn() -> usize) -> u64 {
    let started = Instant::now();
    let mut sum = 0;
    for _ in 0..REPEATS {
        sum += implementation();
    }
    black_box(sum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
