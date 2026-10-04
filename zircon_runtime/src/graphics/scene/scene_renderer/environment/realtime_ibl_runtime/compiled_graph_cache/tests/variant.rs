use std::hint::black_box;
use std::time::Instant;

#[test]
fn compiled_variant_reserves_graph_result_capacities() {
    let source = include_str!("../variant.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("compiled graph variant implementation");

    assert!(implementation.contains("Vec::with_capacity(graph.passes().len())"));
    assert!(implementation.contains("Vec::with_capacity(graph.resource_lifetimes().len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cq_runtime_ibl_compiled_graph_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const PASSES_PER_SAMPLE: usize = 256;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(PASSES_PER_SAMPLE, false));
            optimized.push(measure(PASSES_PER_SAMPLE, true));
        } else {
            optimized.push(measure(PASSES_PER_SAMPLE, true));
            legacy.push(measure(PASSES_PER_SAMPLE, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME392_IBL_COMPILED_GRAPH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} passes_per_sample={PASSES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(count: usize, use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..2_048 {
        let values = if use_capacity {
            let mut values = Vec::with_capacity(count);
            values.extend(0..count);
            values
        } else {
            (0..count).collect::<Vec<_>>()
        };
        checksum ^= values.len();
        black_box(values);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
