use std::hint::black_box;
use std::time::Instant;

use super::callback_stack_contains;

const CALLBACK_DEPTH: usize = 512;
const LOOKUPS_PER_SAMPLE: usize = 16_384;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_fv_runtime478_callback_stack_lookup_preserves_nested_membership() {
    let callbacks = [11, 23, 37, 41];

    assert!(callback_stack_contains(&callbacks, 41));
    assert!(callback_stack_contains(&callbacks, 23));
    assert!(!callback_stack_contains(&callbacks, 43));
    assert!(!callback_stack_contains(&[], 11));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fv_runtime478_reverse_callback_stack_lookup_benchmark() {
    let callbacks = (0..CALLBACK_DEPTH).collect::<Vec<_>>();
    let active_identity = CALLBACK_DEPTH - 1;
    for _ in 0..4 {
        black_box(measure_lookups(&callbacks, active_identity, false));
        black_box(measure_lookups(&callbacks, active_identity, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_lookups(&callbacks, active_identity, false));
            optimized_samples.push(measure_lookups(&callbacks, active_identity, true));
        } else {
            optimized_samples.push(measure_lookups(&callbacks, active_identity, true));
            legacy_samples.push(measure_lookups(&callbacks, active_identity, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME478_REVERSE_CALLBACK_STACK_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} callback_depth={CALLBACK_DEPTH} lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_comparisons_per_top_hit={CALLBACK_DEPTH} optimized_comparisons_per_top_hit=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 25 / 100);
}

fn measure_lookups(callbacks: &[usize], identity: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let found = if optimized {
            callback_stack_contains(black_box(callbacks), black_box(identity))
        } else {
            black_box(callbacks).contains(&black_box(identity))
        };
        black_box(found);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
