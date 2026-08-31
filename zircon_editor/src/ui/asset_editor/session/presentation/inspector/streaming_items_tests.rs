use std::hint::black_box;
use std::time::Instant;

use super::*;

const ITEM_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hj_editor202_preserves_inspector_item_extension() {
    let mut target = vec!["reflection".to_string()];
    let source = ["width".to_string(), "height".to_string()];

    extend_cloned_values(&mut target, &source);

    assert_eq!(
        target.iter().map(String::as_str).collect::<Vec<_>>(),
        ["reflection", "width", "height"]
    );
    assert_eq!(
        source.iter().map(String::as_str).collect::<Vec<_>>(),
        ["width", "height"]
    );
}

#[test]
fn optimization_batch_20260826hj_editor202_streams_inspector_item_clones() {
    let source = include_str!("../inspector.rs");
    assert!(source.contains("extend_cloned_values(&mut inspector_items, &widget_prop_state_items)"));
    assert!(source.contains("target.extend_from_slice(source)"));
    assert!(!source.contains("inspector_items.extend(widget_prop_state_items.clone())"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hj_editor202_streaming_inspector_item_release_benchmark() {
    let source = (0..ITEM_COUNT)
        .map(|value| value as u64)
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(ITEM_COUNT);
    let mut optimized = Vec::with_capacity(ITEM_COUNT);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                legacy.clear();
                legacy_extend_cloned_values(black_box(&mut legacy), black_box(&source));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                optimized.clear();
                extend_cloned_values(black_box(&mut optimized), black_box(&source));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR202_STREAMING_INSPECTOR_ITEM_BENCH_V1 \
         item_count={ITEM_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_extend_cloned_values<T: Clone>(target: &mut Vec<T>, source: &[T]) {
    target.extend(source.to_vec());
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
