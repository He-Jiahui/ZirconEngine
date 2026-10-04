use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const UPDATED_COUNT: usize = 32_768;
const REMOVED_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iv_runtime632_reserves_targeted_catalog_membership() {
    let source = include_str!("../../catalog_input_generation.rs");
    let targeted_body = source
        .split("pub(crate) fn publish_targeted")
        .nth(1)
        .expect("targeted catalog publication remains present")
        .split("pub(crate) fn publish_metadata")
        .next()
        .expect("targeted catalog publication remains bounded");

    assert!(targeted_body.contains("let updated_records = updated_records.into_iter();"));
    assert!(targeted_body.contains("let removed_ids = removed_ids.into_iter();"));
    assert!(targeted_body.contains("HashSet::with_capacity(updated_capacity)"));
    assert!(targeted_body.contains("let mut touched_ids = HashSet::with_capacity("));
    assert!(targeted_body.contains("updated_capacity.saturating_add(removed_capacity)"));
    assert!(!targeted_body.contains("let mut updated_ids = HashSet::new();"));
    assert!(!targeted_body.contains("let mut touched_ids = HashSet::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iv_runtime632_preallocated_catalog_membership_benchmark() {
    for _ in 0..4 {
        black_box(measure_membership(false));
        black_box(measure_membership(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_membership(false));
            preallocated_samples.push(measure_membership(true));
        } else {
            preallocated_samples.push(measure_membership(true));
            unreserved_samples.push(measure_membership(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME632_PREALLOCATED_TARGETED_CATALOG_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} updated_count={UPDATED_COUNT} removed_count={REMOVED_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_membership(preallocated: bool) -> u128 {
    let mut updated = if preallocated {
        HashSet::with_capacity(UPDATED_COUNT)
    } else {
        HashSet::new()
    };
    let mut touched = if preallocated {
        HashSet::with_capacity(UPDATED_COUNT.saturating_add(REMOVED_COUNT))
    } else {
        HashSet::new()
    };
    let started = Instant::now();
    for id in black_box(0..REMOVED_COUNT) {
        black_box(touched.insert(id));
    }
    for id in black_box(REMOVED_COUNT..REMOVED_COUNT.saturating_add(UPDATED_COUNT)) {
        black_box(touched.insert(id));
        black_box(updated.insert(id));
    }
    black_box((updated, touched));
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
