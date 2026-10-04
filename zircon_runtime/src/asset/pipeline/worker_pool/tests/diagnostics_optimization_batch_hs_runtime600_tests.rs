use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::diagnostics::DiagnosticStore;

use super::{
    AssetWorkerPoolFrameDiagnostics, AssetWorkerThreadBudgetSource,
    ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC, ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC,
    ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC, ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
};

const SAMPLE_PAIRS: usize = 17;
const ITERATIONS_PER_SAMPLE: usize = 8_192;

#[test]
fn optimization_batch_hs_runtime600_static_asset_metrics_preserve_snapshot() {
    let diagnostics = fixture();
    let mut legacy = DiagnosticStore::new(2);
    let mut optimized = DiagnosticStore::new(2);

    record_legacy(&diagnostics, &mut legacy, 7);
    diagnostics.record_diagnostics(&mut optimized, 7);

    assert_eq!(optimized.snapshot(), legacy.snapshot());
}

#[test]
fn optimization_batch_hs_runtime600_asset_producers_use_static_diagnostics() {
    let source = include_str!("../diagnostics.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("asset worker diagnostics production");

    assert!(!production.contains("store.record("));
    assert!(production.matches("store.record_static(").count() >= 6);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hs_runtime600_static_asset_metrics_p95() {
    const MARKER: &str = "RUNTIME600_ASSET_WORKER_STATIC_DIAGNOSTICS_BENCH_V1";
    let diagnostics = fixture();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&diagnostics, false));
            optimized.push(measure(&diagnostics, true));
        } else {
            optimized.push(measure(&diagnostics, true));
            legacy.push(measure(&diagnostics, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} samples={SAMPLE_PAIRS} iterations={ITERATIONS_PER_SAMPLE} metrics=4"
    );
    assert!(
        ratio <= 0.55,
        "{MARKER} expected static diagnostic ratio <= 0.55, got {ratio:.4}"
    );
}

fn fixture() -> AssetWorkerPoolFrameDiagnostics {
    AssetWorkerPoolFrameDiagnostics {
        thread_budget_source: AssetWorkerThreadBudgetSource::TaskPoolIo,
        budgeted_threads: 4,
        in_flight: 9,
        completed_delta: 7,
        failed_delta: 2,
    }
}

fn record_legacy(
    diagnostics: &AssetWorkerPoolFrameDiagnostics,
    store: &mut DiagnosticStore,
    frame_index: u64,
) {
    store.record(
        ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC,
        frame_index,
        diagnostics.in_flight as f64,
        Some("request"),
        ["asset", "worker"],
    );
    store.record(
        ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC,
        frame_index,
        diagnostics.budgeted_threads as f64,
        Some("thread"),
        [
            "asset",
            "worker",
            "budget",
            diagnostics.thread_budget_source.as_str(),
        ],
    );
    store.record(
        ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC,
        frame_index,
        diagnostics.completed_delta as f64,
        Some("request"),
        ["asset", "worker", "frame"],
    );
    store.record(
        ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC,
        frame_index,
        diagnostics.failed_delta as f64,
        Some("request"),
        ["asset", "worker", "frame"],
    );
}

fn measure(diagnostics: &AssetWorkerPoolFrameDiagnostics, optimized: bool) -> Duration {
    let mut store = DiagnosticStore::new(1);
    let started = Instant::now();
    for frame_index in 0..ITERATIONS_PER_SAMPLE as u64 {
        if optimized {
            diagnostics.record_diagnostics(&mut store, frame_index);
        } else {
            record_legacy(diagnostics, &mut store, frame_index);
        }
    }
    let elapsed = started.elapsed();
    black_box(store.current_snapshot());
    elapsed
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    let index = (values.len() - 1) * percentile / 100;
    values[index]
}
