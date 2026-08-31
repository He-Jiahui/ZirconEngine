use std::hint::black_box;
use std::time::Instant;

use super::push_unique_with_recent_fast_path;

const SAMPLE_PAIRS: usize = 31;
const CHECKS_PER_SAMPLE: usize = 5_000;
const EXISTING_ITEM_COUNT: usize = 512;

#[test]
fn optimization_batch_20260829an_runtime314_recent_and_earlier_duplicates_preserve_first_order() {
    let mut values = vec![1, 2, 3, 4];

    push_unique_with_recent_fast_path(&mut values, 4);
    push_unique_with_recent_fast_path(&mut values, 2);
    push_unique_with_recent_fast_path(&mut values, 5);

    assert_eq!(values, [1, 2, 3, 4, 5]);
}

#[test]
fn optimization_batch_20260829an_runtime314_readiness_pushes_share_the_recent_fast_path() {
    let source = include_str!("../readiness_report.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("material readiness production source");
    let helper = production
        .split("fn push_unique_with_recent_fast_path")
        .nth(1)
        .expect("recent duplicate helper");
    let last_check = helper.find("items.last()").expect("last item check");
    let full_scan = helper.find("items.contains(").expect("fallback full scan");

    assert!(last_check < full_scan);
    assert_eq!(
        production
            .matches("push_unique_with_recent_fast_path(")
            .count(),
        3
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829an_runtime314_recent_material_diagnostic_dedup_bench() {
    let mut legacy = existing_items();
    let mut optimized = existing_items();
    legacy_push_unique(&mut legacy, EXISTING_ITEM_COUNT - 1);
    push_unique_with_recent_fast_path(&mut optimized, EXISTING_ITEM_COUNT - 1);
    assert_eq!(optimized, legacy);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME314_RECENT_MATERIAL_DIAGNOSTIC_DEDUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} existing_items={EXISTING_ITEM_COUNT} \
legacy_comparisons_per_check=512 optimized_comparisons_per_check=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn existing_items() -> Vec<usize> {
    (0..EXISTING_ITEM_COUNT).collect()
}

fn legacy_push_unique(items: &mut Vec<usize>, item: usize) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn measure(optimized: bool) -> u128 {
    let mut items = existing_items();
    let item = EXISTING_ITEM_COUNT - 1;
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            push_unique_with_recent_fast_path(black_box(&mut items), black_box(item));
        } else {
            legacy_push_unique(black_box(&mut items), black_box(item));
        }
    }
    black_box(items.len());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
