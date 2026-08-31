use std::hint::black_box;
use std::time::Instant;

use super::{metric_value, PickingDebugMetric, PickingDebugMetricKind};

const SAMPLE_PAIRS: usize = 31;
const LOOKUPS_PER_SAMPLE: usize = 250_000;

#[test]
fn optimization_batch_20260829ap_runtime316_expected_metric_slots_preserve_lookup_semantics() {
    let mut metrics = metrics();

    assert_eq!(
        metric_value(&metrics, PickingDebugMetricKind::BlockedPointers),
        Some(60)
    );
    metrics.swap(0, 5);
    assert_eq!(
        metric_value(&metrics, PickingDebugMetricKind::BlockedPointers),
        Some(60)
    );
    assert_eq!(
        metric_value(&metrics, PickingDebugMetricKind::Pointers),
        Some(10)
    );
}

#[test]
fn optimization_batch_20260829ap_runtime316_metric_lookup_uses_expected_slot_before_fallback() {
    let source = include_str!("../debug_feed.rs");
    let lookup = source
        .split("fn metric_value")
        .nth(1)
        .expect("metric value lookup")
        .split("pub struct PickingDebugMetric")
        .next()
        .expect("metric value lookup body");

    assert!(lookup.contains("expected_metric_index(kind)"));
    assert!(lookup.contains("metrics.get(expected_index)"));
    assert!(lookup.contains("or_else(|| metrics.iter().find"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ap_runtime316_indexed_picking_debug_metric_lookups_bench() {
    let metrics = metrics();
    let kind = PickingDebugMetricKind::BlockedPointers;
    assert_eq!(
        metric_value(&metrics, kind),
        legacy_metric_value(&metrics, kind)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&metrics, kind, false));
            optimized_samples.push(measure(&metrics, kind, true));
        } else {
            optimized_samples.push(measure(&metrics, kind, true));
            legacy_samples.push(measure(&metrics, kind, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME316_INDEXED_PICKING_DEBUG_METRIC_LOOKUPS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_worst_case_comparisons_per_lookup=6 \
optimized_expected_slot_checks_per_lookup=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn metrics() -> Vec<PickingDebugMetric> {
    use PickingDebugMetricKind::{
        BackendOutputs, BlockedPointers, HoveredHits, Pointers, RawHits, Rays,
    };

    vec![
        PickingDebugMetric::new(Pointers, 10),
        PickingDebugMetric::new(Rays, 20),
        PickingDebugMetric::new(BackendOutputs, 30),
        PickingDebugMetric::new(RawHits, 40),
        PickingDebugMetric::new(HoveredHits, 50),
        PickingDebugMetric::new(BlockedPointers, 60),
    ]
}

fn legacy_metric_value(
    metrics: &[PickingDebugMetric],
    kind: PickingDebugMetricKind,
) -> Option<usize> {
    metrics
        .iter()
        .find(|metric| metric.kind == kind)
        .map(|metric| metric.value)
}

fn measure(metrics: &[PickingDebugMetric], kind: PickingDebugMetricKind, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let value = if optimized {
            metric_value(black_box(metrics), black_box(kind))
        } else {
            legacy_metric_value(black_box(metrics), black_box(kind))
        }
        .expect("benchmark metric");
        checksum = checksum.wrapping_add(value);
    }
    black_box(checksum);
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
