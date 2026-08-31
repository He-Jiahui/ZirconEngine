use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::percentile_ms;

const MARKER: &str = "RUNTIME237_CONFIG_PERCENTILE_STACK_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 16_384;

#[test]
fn optimization_batch_20260826gq_runtime237_percentile_preserves_empty_and_wrapped_samples() {
    assert_eq!(percentile_ms(&VecDeque::new(), 95), 0.0);

    let mut samples = (0_u64..64).collect::<VecDeque<_>>();
    for next in 64_u64..80 {
        assert!(samples.pop_front().is_some());
        samples.push_back(next);
    }

    assert_eq!(percentile_ms(&samples, 50), 0.000_047);
    assert_eq!(percentile_ms(&samples, 95), 0.000_076);
}

#[test]
fn optimization_batch_20260826gq_runtime237_percentile_sorts_a_fixed_stack_buffer() {
    let source = include_str!("../state.rs");
    assert!(source.contains("[0_u64; MAX_FLUSH_LATENCY_SAMPLES]"));
    assert!(source.contains("samples.as_slices()"));
    assert!(!source.contains("samples.iter().copied().collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gq_runtime237_config_percentile_stack_buffer_bench() {
    let samples = (0_u64..64).collect::<VecDeque<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy_first = pair % 2 == 0;
        if legacy_first {
            legacy_samples.push(measure(&samples, legacy_percentile_ms));
            optimized_samples.push(measure(&samples, percentile_ms));
        } else {
            optimized_samples.push(measure(&samples, percentile_ms));
            legacy_samples.push(measure(&samples, legacy_percentile_ms));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "stack percentile must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_percentile_ms(samples: &VecDeque<u64>, percentile: usize) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut ordered = samples.iter().copied().collect::<Vec<_>>();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100).saturating_sub(1);
    ordered[rank] as f64 / 1_000_000.0
}

fn measure(samples: &VecDeque<u64>, implementation: fn(&VecDeque<u64>, usize) -> f64) -> u64 {
    let started = Instant::now();
    for _ in 0..REPEATS {
        black_box(implementation(black_box(samples), 95));
    }
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
