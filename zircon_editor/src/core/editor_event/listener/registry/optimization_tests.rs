use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use crate::core::editor_event::EditorEventListenerDescriptor;

use super::EditorEventListenerRegistry;

const LISTENER_COUNT: usize = 1_024;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 8;

fn registry_fixture() -> EditorEventListenerRegistry {
    let mut registry = EditorEventListenerRegistry::default();
    for index in 0..LISTENER_COUNT {
        registry
            .register(format!("listener-{index:04}"), format!("Listener {index}"))
            .expect("fixture listener registration");
    }
    registry
}

fn legacy_listener_projection(
    registry: &EditorEventListenerRegistry,
) -> Vec<EditorEventListenerDescriptor> {
    registry
        .listener_order
        .iter()
        .filter_map(|listener_id| registry.listeners.get(listener_id))
        .map(|listener| listener.descriptor.clone())
        .collect()
}

#[test]
fn editor49_listener_registry_capacity_preserves_order_and_routes() {
    let registry = registry_fixture();
    let listeners = registry.listeners();
    assert_eq!(listeners.len(), LISTENER_COUNT);
    assert_eq!(listeners[0].listener_id, "listener-0000");
    assert_eq!(listeners[LISTENER_COUNT - 1].listener_id, "listener-1023");
    assert_eq!(registry.delivery_routes().len(), LISTENER_COUNT);
}

#[test]
fn editor49_listener_registry_capacity_source_contract() {
    let source = include_str!("../registry.rs");
    assert!(source.contains("Vec::with_capacity(self.listener_order.len())"));
    assert!(source.contains("let mut routes = Vec::with_capacity"));
    assert!(!source.contains(".collect::<Vec<_>>()"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor49_listener_registry_capacity_bench() {
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let registry = registry_fixture();
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_listener_projection(&registry));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let registry = registry_fixture();
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(registry.listeners());
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let percentile_95 = |mut samples: Vec<u128>| {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    };
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "EDITOR49_LISTENER_REGISTRY_CAPACITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} listeners={} route_capacity_bound={} start_capacity_reduction=0->{}",
        legacy_p95,
        optimized_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        LISTENER_COUNT,
        LISTENER_COUNT,
        LISTENER_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized listener projection p95 should be at most 95% of legacy p95"
    );
}

#[test]
fn optimization_batch_hi_editor592_unchanged_configuration_reuses_route_snapshot() {
    let mut registry = registry_fixture();
    let initial = registry.delivery_routes();

    registry
        .set_enabled("listener-1023", true)
        .expect("unchanged enabled state");
    assert!(Arc::ptr_eq(&initial, &registry.delivery_routes()));

    let filter = super::super::EditorEventListenerFilter::default().failures_only();
    registry
        .set_filter("listener-1023", filter.clone())
        .expect("initial filter");
    let filtered = registry.delivery_routes();
    registry
        .set_filter("listener-1023", filter)
        .expect("unchanged filter");
    assert!(Arc::ptr_eq(&filtered, &registry.delivery_routes()));

    registry
        .clear_filter("listener-1023")
        .expect("clear filter");
    let cleared = registry.delivery_routes();
    registry
        .clear_filter("listener-1023")
        .expect("unchanged clear filter");
    assert!(Arc::ptr_eq(&cleared, &registry.delivery_routes()));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_editor592_unchanged_enabled_route_rebuild_benchmark() {
    const BENCH_SAMPLES: usize = 17;
    const BENCH_ITERATIONS: usize = 64;
    let mut registry = registry_fixture();
    let mut legacy_samples = Vec::with_capacity(BENCH_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCH_SAMPLES);
    for sample in 0..BENCH_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy_noop_enabled(&mut registry, BENCH_ITERATIONS));
            optimized_samples.push(measure_optimized_noop_enabled(
                &mut registry,
                BENCH_ITERATIONS,
            ));
        } else {
            optimized_samples.push(measure_optimized_noop_enabled(
                &mut registry,
                BENCH_ITERATIONS,
            ));
            legacy_samples.push(measure_legacy_noop_enabled(&mut registry, BENCH_ITERATIONS));
        }
    }
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "EDITOR592_UNCHANGED_LISTENER_ROUTE_BENCH_V1 sample_pairs={BENCH_SAMPLES} iterations={BENCH_ITERATIONS} listeners={LISTENER_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}"
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(20));
}

fn measure_legacy_noop_enabled(
    registry: &mut EditorEventListenerRegistry,
    iterations: usize,
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        registry
            .listener_mut(black_box("listener-1023"))
            .expect("fixture listener")
            .descriptor
            .enabled = true;
        registry.rebuild_delivery_routes();
        black_box(registry.delivery_routes());
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized_noop_enabled(
    registry: &mut EditorEventListenerRegistry,
    iterations: usize,
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(
            registry
                .set_enabled(black_box("listener-1023"), true)
                .expect("fixture listener"),
        );
    }
    started.elapsed().as_nanos().max(1)
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}
