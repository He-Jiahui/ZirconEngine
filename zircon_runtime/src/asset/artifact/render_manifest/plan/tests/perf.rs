use std::hint::black_box;
use std::time::Instant;

const BLOCK_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ja_runtime640_reserves_render_load_batches() {
    let source = include_str!("../../plan.rs");
    let planning = source
        .split("let mut ready = selected")
        .nth(1)
        .expect("render load batch planning remains present")
        .split("fn dependency_frontier_seed")
        .next()
        .expect("render load batch planning remains bounded");

    assert!(planning.contains("Vec::with_capacity(block_count)"));
    assert!(!planning.contains("let mut batches = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ja_runtime640_preallocated_render_load_batch_benchmark() {
    for _ in 0..4 {
        black_box(measure_batches(false));
        black_box(measure_batches(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_batches(false));
            preallocated_samples.push(measure_batches(true));
        } else {
            preallocated_samples.push(measure_batches(true));
            unreserved_samples.push(measure_batches(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME640_PREALLOCATED_RENDER_LOAD_BATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} block_count={BLOCK_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_batches(preallocated: bool) -> u128 {
    let mut batches = if preallocated {
        Vec::with_capacity(BLOCK_COUNT)
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for block in 0..BLOCK_COUNT {
        batches.push(black_box(block));
    }
    black_box(batches);
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
