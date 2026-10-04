use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const KEYS_PER_SAMPLE: usize = 262_144;

#[test]
fn product_identity_is_generation_scoped_and_backend_neutral() {
    let first = RenderViewportProduct::new(RenderViewportHandle::new(7), 640, 360, 3);
    let next = RenderViewportProduct::new(RenderViewportHandle::new(7), 640, 360, 4);

    assert_eq!(first.resource_key(), "viewport:7:3");
    assert_ne!(first.resource_key(), next.resource_key());
    assert!(first.is_valid());
}

#[test]
fn optimization_batch_ew_runtime455_preserves_viewport_resource_key_bytes() {
    for (viewport, generation) in [(0, 0), (1, 9), (42, 10), (u32::MAX as u64, u64::MAX)] {
        assert_eq!(
            viewport_resource_key(viewport, generation),
            format!("viewport:{viewport}:{generation}")
        );
    }

    let production = include_str!("../viewport_product.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(!production.contains("resource_key: format!"));
    assert!(production.contains("String::with_capacity(capacity)"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ew_runtime455_direct_viewport_resource_key_benchmark() {
    for _ in 0..4 {
        black_box(measure_legacy_keys());
        black_box(measure_direct_keys());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy_keys());
            optimized_samples.push(measure_direct_keys());
        } else {
            optimized_samples.push(measure_direct_keys());
            legacy_samples.push(measure_legacy_keys());
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure_legacy_keys() -> u128 {
    measure_keys(|viewport, generation| format!("viewport:{viewport}:{generation}"))
}

fn measure_direct_keys() -> u128 {
    measure_keys(viewport_resource_key)
}

fn measure_keys(mut build: impl FnMut(u64, u64) -> String) -> u128 {
    let started = Instant::now();
    let mut total_len = 0_usize;
    for index in 0..KEYS_PER_SAMPLE {
        let viewport = black_box((index % 32) as u64);
        let generation = black_box(10_000_000_u64 + index as u64);
        let key = build(viewport, generation);
        total_len += black_box(key.len());
        black_box(key);
    }
    black_box(total_len);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME455_DIRECT_VIEWPORT_RESOURCE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} keys_per_sample={KEYS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "direct viewport resource key construction must reduce P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
