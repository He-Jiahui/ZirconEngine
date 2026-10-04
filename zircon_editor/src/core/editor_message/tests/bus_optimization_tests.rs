use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::*;

const SAMPLE_COUNT: usize = 17;
const TARGET_COUNT: usize = 1_024;
const ITERATIONS: usize = 64;

fn fixture() -> (
    BTreeMap<EditorSubscriberId, Arc<Mutex<EditorMessageInbox>>>,
    BTreeSet<EditorSubscriberId>,
) {
    let subscribers = (0..TARGET_COUNT as u64)
        .map(EditorSubscriberId::new)
        .collect::<BTreeSet<_>>();
    let inboxes = subscribers
        .iter()
        .copied()
        .map(|subscriber| {
            (
                subscriber,
                Arc::new(Mutex::new(EditorMessageInbox::new(
                    EditorMessageInboxLimits::default(),
                ))),
            )
        })
        .collect();
    (inboxes, subscribers)
}

fn legacy_dispatch_targets(
    inboxes: &BTreeMap<EditorSubscriberId, Arc<Mutex<EditorMessageInbox>>>,
    subscribers: &BTreeSet<EditorSubscriberId>,
) -> Vec<EditorMessageDispatchTarget> {
    let targets = subscribers.iter().copied().collect::<Vec<_>>();
    targets
        .into_iter()
        .filter_map(|subscriber| {
            inboxes
                .get(&subscriber)
                .cloned()
                .map(|inbox| EditorMessageDispatchTarget { subscriber, inbox })
        })
        .collect()
}

fn delivery_ids(reserve_exact: bool) -> Vec<EditorSubscriberId> {
    let mut delivered = Vec::new();
    if reserve_exact {
        delivered.reserve_exact(TARGET_COUNT);
    }
    delivered.extend((0..TARGET_COUNT as u64).map(EditorSubscriberId::new));
    delivered
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn optimization_batch_hi_editor595_dispatch_target_single_buffer_preserves_order() {
    let (inboxes, subscribers) = fixture();
    let legacy = legacy_dispatch_targets(&inboxes, &subscribers);
    let optimized = collect_dispatch_targets(&inboxes, subscribers.iter().copied());

    assert_eq!(optimized.len(), legacy.len());
    for (optimized, legacy) in optimized.iter().zip(&legacy) {
        assert_eq!(optimized.subscriber, legacy.subscriber);
        assert!(Arc::ptr_eq(&optimized.inbox, &legacy.inbox));
    }
}

#[test]
fn optimization_batch_hi_editor595_publish_and_broadcast_skip_id_buffer() {
    let source = include_str!("../bus.rs");
    let publish = source
        .split("pub(super) fn prepare_publish")
        .nth(1)
        .expect("publish implementation")
        .split("pub(super) fn prepare_broadcast")
        .next()
        .unwrap();
    let broadcast = source
        .split("pub(super) fn prepare_broadcast")
        .nth(1)
        .expect("broadcast implementation")
        .split("pub(super) fn prepare_request")
        .next()
        .unwrap();

    assert!(publish.contains("collect_dispatch_targets"));
    assert!(broadcast.contains("collect_dispatch_targets"));
    assert!(!publish.contains("collect::<Vec<_>>()"));
    assert!(!broadcast.contains("collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_editor595_dispatch_target_single_buffer_bench() {
    let (inboxes, subscribers) = fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_dispatch_targets(
                    black_box(&inboxes),
                    black_box(&subscribers),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(collect_dispatch_targets(
                    black_box(&inboxes),
                    black_box(subscribers.iter().copied()),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "EDITOR595_MESSAGE_TARGET_SINGLE_BUFFER_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} targets={} target_buffers=2->1 exact_capacity=true",
        legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS, TARGET_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(90),
        "single-buffer target projection P95 must be at most 90% of legacy P95"
    );
}

#[test]
fn optimization_batch_hi_editor596_delivery_report_capacity_preserves_order() {
    let legacy = delivery_ids(false);
    let optimized = delivery_ids(true);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.capacity(), TARGET_COUNT);
}

#[test]
fn optimization_batch_hi_editor596_lossless_reserves_delivered_after_admission() {
    let source = include_str!("../bus.rs");
    let lossless = source
        .split("fn dispatch_lossless")
        .nth(1)
        .expect("lossless dispatch")
        .split("fn dispatch_best_effort")
        .next()
        .unwrap();
    let best_effort = source
        .split("fn dispatch_best_effort")
        .nth(1)
        .expect("best-effort dispatch")
        .split("fn record_enqueue_outcome")
        .next()
        .unwrap();
    let backpressure_return = lossless
        .find("if !report.backpressured.is_empty()")
        .expect("lossless admission barrier");
    let delivered_reserve = lossless
        .find("report.delivered.reserve_exact(self.targets.len())")
        .expect("lossless delivered capacity");

    assert!(backpressure_return < delivered_reserve);
    assert!(!best_effort.contains("report.delivered.reserve_exact(self.targets.len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_editor596_delivery_report_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(delivery_ids(false));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(delivery_ids(true));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "EDITOR596_MESSAGE_DELIVERY_REPORT_CAPACITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} targets={} delivered_growth=geometric->exact",
        legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS, TARGET_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "exact delivery-report capacity P95 must be at most 80% of growable delivery P95"
    );
}
