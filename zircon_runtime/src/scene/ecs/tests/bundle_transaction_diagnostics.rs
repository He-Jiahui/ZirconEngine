use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;
use crate::scene::ecs::{ChangeDetectionScanStats, QueryStateCacheStats};

use super::BundleTransactionDiagnostics;

#[test]
fn transaction_diagnostics_derive_intermediate_signatures_from_actual_assignments() {
    let mut diagnostics = BundleTransactionDiagnostics::default();

    diagnostics.record_commit(true, 3, 0, 0, 0);

    assert_eq!(diagnostics.committed_transactions, 1);
    assert_eq!(diagnostics.final_archetype_transitions, 1);
    assert_eq!(diagnostics.intermediate_signatures, 2);
}

#[test]
fn optimization_batch_hv_runtime604_static_ecs_metrics_preserve_snapshot() {
    let (bundle, change, query) = fixtures();
    let mut legacy = DiagnosticStore::new(2);
    let mut optimized = DiagnosticStore::new(2);

    record_legacy(&bundle, &change, &query, &mut legacy, 13);
    bundle.record_diagnostics(&mut optimized, 13);
    change.record_diagnostics(&mut optimized, 13);
    query.record_diagnostics(&mut optimized, 13);

    assert_eq!(optimized.snapshot(), legacy.snapshot());
}

#[test]
fn optimization_batch_hv_runtime604_ecs_producers_use_static_diagnostics() {
    for (source, expected_calls) in [
        (include_str!("../bundle_transaction_diagnostics.rs"), 1),
        (include_str!("../change_detection/stats.rs"), 1),
        (include_str!("../query/query_state/stats.rs"), 1),
    ] {
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("ECS diagnostic production");
        assert!(!production.contains("store.record("));
        assert_eq!(
            production.matches("store.record_static(").count(),
            expected_calls
        );
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hv_runtime604_static_ecs_metrics_p95() {
    const MARKER: &str = "RUNTIME604_ECS_STATIC_DIAGNOSTICS_BENCH_V1";
    const SAMPLE_PAIRS: usize = 17;
    const ITERATIONS: usize = 4_096;
    let (bundle, change, query) = fixtures();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&bundle, &change, &query, ITERATIONS, false));
            optimized.push(measure(&bundle, &change, &query, ITERATIONS, true));
        } else {
            optimized.push(measure(&bundle, &change, &query, ITERATIONS, true));
            legacy.push(measure(&bundle, &change, &query, ITERATIONS, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} samples={SAMPLE_PAIRS} iterations={ITERATIONS} metrics=18"
    );
    assert!(
        ratio <= 0.50,
        "{MARKER} expected static diagnostic ratio <= 0.50, got {ratio:.4}"
    );
}

fn fixtures() -> (
    BundleTransactionDiagnostics,
    ChangeDetectionScanStats,
    QueryStateCacheStats,
) {
    (
        BundleTransactionDiagnostics {
            committed_transactions: 11,
            final_archetype_transitions: 10,
            intermediate_signatures: 1,
            component_storage_moves: 17,
            lifecycle_events: 23,
            staged_value_allocations: 5,
        },
        ChangeDetectionScanStats {
            scanned_marks: 37,
            added_matches: 7,
            changed_matches: 13,
        },
        QueryStateCacheStats {
            cache_hits: 101,
            cache_misses: 3,
            cache_rebuilds: 2,
            archetype_plan_compilations: 5,
            archetype_component_membership_checks: 71,
            table_column_slot_bindings: 29,
            sparse_component_bindings: 19,
            candidate_entity_count: 257,
            matched_entity_count: 89,
            ..QueryStateCacheStats::default()
        },
    )
}

fn record_legacy(
    bundle: &BundleTransactionDiagnostics,
    change: &ChangeDetectionScanStats,
    query: &QueryStateCacheStats,
    store: &mut DiagnosticStore,
    frame_index: u64,
) {
    for (path, value) in bundle.diagnostic_values() {
        store.record(path, frame_index, value, Some("count"), ["ecs", "bundle"]);
    }
    for (path, value) in change.diagnostic_values() {
        store.record(
            path,
            frame_index,
            value,
            Some("count"),
            ["ecs", "change_detection"],
        );
    }
    for (path, value) in query.diagnostic_values() {
        store.record(path, frame_index, value, Some("count"), ["ecs", "query"]);
    }
}

fn measure(
    bundle: &BundleTransactionDiagnostics,
    change: &ChangeDetectionScanStats,
    query: &QueryStateCacheStats,
    iterations: usize,
    optimized: bool,
) -> Duration {
    let mut store = DiagnosticStore::new(1);
    let started = Instant::now();
    for frame_index in 0..iterations as u64 {
        if optimized {
            bundle.record_diagnostics(&mut store, frame_index);
            change.record_diagnostics(&mut store, frame_index);
            query.record_diagnostics(&mut store, frame_index);
        } else {
            record_legacy(bundle, change, query, &mut store, frame_index);
        }
    }
    let elapsed = started.elapsed();
    black_box(store.current_snapshot());
    elapsed
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    values[(values.len() - 1) * percentile / 100]
}
