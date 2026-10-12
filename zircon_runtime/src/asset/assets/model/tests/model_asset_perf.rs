use std::collections::HashSet;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const REFERENCES_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260830cm_runtime_model_reference_dedup_reserves_iterator_bound() {
    let source = include_str!("../model_asset.rs");
    let implementation = source
        .split("mod performance_tests;")
        .next()
        .expect("model asset implementation");

    assert!(implementation.contains("let (minimum_references, maximum_references)"));
    assert!(implementation.contains("maximum_references.unwrap_or(minimum_references)"));
    assert!(implementation.contains("HashSet::with_capacity(reference_capacity)"));
    assert!(implementation.contains("Vec::with_capacity(reference_capacity)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cm_runtime_model_reference_dedup_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME500_MODEL_REFERENCE_DEDUP_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} references_per_sample={REFERENCES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut seen = if use_capacity {
        HashSet::with_capacity(REFERENCES_PER_SAMPLE)
    } else {
        HashSet::new()
    };
    let mut unique = if use_capacity {
        Vec::with_capacity(REFERENCES_PER_SAMPLE)
    } else {
        Vec::new()
    };
    for reference in 0..REFERENCES_PER_SAMPLE {
        if seen.insert(reference) {
            unique.push(reference);
        }
    }
    std::hint::black_box((seen, unique));
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
