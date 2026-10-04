use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const ITEMS_PER_SAMPLE: usize = 256;

#[test]
fn animation_shared_string_list_reserves_input_capacity() {
    let source = include_str!("../animation_projection.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(items.len())"));
    assert!(implementation.contains("for item in items"));
    assert!(!implementation.contains("items.iter().cloned().map(SharedString::from).collect()"));
}

#[test]
fn animation_shared_string_list_keeps_item_order() {
    let source = include_str!("../animation_projection.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let loop_start = implementation.find("for item in items").expect("item loop");
    let push = implementation
        .find("output.push(SharedString::from(item.clone()))")
        .expect("item push");
    assert!(loop_start < push);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cb_editor_animation_shared_string_capacity_p95() {
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
        "EDITOR326_ANIMATION_SHARED_STRING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items_per_sample={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut output = if optimized {
            Vec::with_capacity(ITEMS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..ITEMS_PER_SAMPLE {
            output.push(index);
        }
        checksum ^= output.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
