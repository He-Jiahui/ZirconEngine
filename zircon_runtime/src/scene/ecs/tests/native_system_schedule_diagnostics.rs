use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn native_system_schedule_diagnostics_record_conflicts_latency_and_utilization() {
    let mut diagnostics = NativeSystemScheduleDiagnostics::default();
    diagnostics.record_conflicts(3);
    diagnostics.record_main_callback(Duration::from_micros(10), true);
    diagnostics.record_worker_batch(
        &[
            NativeSystemCallbackTiming {
                ready_delay: Duration::from_micros(4),
                callback: Duration::from_micros(20),
            },
            NativeSystemCallbackTiming {
                ready_delay: Duration::from_micros(8),
                callback: Duration::from_micros(30),
            },
        ],
        Duration::from_micros(40),
        2,
        3,
        384,
    );

    assert_eq!(diagnostics.conflict_count(), 3);
    assert_eq!(diagnostics.worker_batch_count(), 1);
    assert_eq!(diagnostics.callback_count(), 3);
    assert_eq!(diagnostics.conservative_world_writer_count(), 1);
    assert_eq!(diagnostics.temporary_control_buffer_count(), 3);
    assert_eq!(diagnostics.temporary_control_buffer_bytes(), 384);
    assert!((diagnostics.ready_delay_ms() - 0.006).abs() < f64::EPSILON);
    assert_eq!(diagnostics.worker_utilization(), 0.625);
    assert!(diagnostics.callback_p95_ms() >= 0.03);

    let mut store = DiagnosticStore::default();
    diagnostics.record_diagnostics(&mut store, 7);
    let snapshot = store.snapshot();
    assert_eq!(snapshot.series.len(), 9);
    assert!(snapshot.series.iter().any(|series| {
        series.path.as_str() == NATIVE_SYSTEM_CONFLICT_COUNT_DIAGNOSTIC
            && series.current == Some(3.0)
    }));
    assert!(snapshot.series.iter().any(|series| {
        series.path.as_str() == NATIVE_SYSTEM_CALLBACK_P95_MS_DIAGNOSTIC
            && series.current.is_some_and(|value| value >= 0.03)
    }));
    assert!(snapshot.series.iter().any(|series| {
        series.path.as_str() == NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_COUNT_DIAGNOSTIC
            && series.current == Some(3.0)
    }));
    assert!(snapshot.series.iter().any(|series| {
        series.path.as_str() == NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_BYTES_DIAGNOSTIC
            && series.current == Some(384.0)
    }));
}

#[test]
fn optimization_batch_hw_runtime606_static_frame_metrics_preserve_snapshot() {
    let diagnostics = representative_diagnostics();
    let mut legacy = DiagnosticStore::new(2);
    let mut optimized = DiagnosticStore::new(2);

    record_legacy(&diagnostics, &mut legacy, 19);
    diagnostics.record_diagnostics(&mut optimized, 19);

    assert_eq!(optimized.snapshot(), legacy.snapshot());
}

#[test]
fn optimization_batch_hw_runtime606_frame_producers_use_static_diagnostics() {
    for (source, expected_calls) in [
        (include_str!("../native_system_schedule_diagnostics.rs"), 1),
        (include_str!("../frame_performance_diagnostics.rs"), 3),
    ] {
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("ECS frame diagnostic production");
        assert!(!production.contains("store.record("));
        assert_eq!(
            production.matches("store.record_static(").count(),
            expected_calls
        );
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hw_runtime606_static_frame_metrics_p95() {
    const MARKER: &str = "RUNTIME606_ECS_FRAME_STATIC_DIAGNOSTICS_BENCH_V1";
    const SAMPLE_PAIRS: usize = 17;
    const ITERATIONS: usize = 8_192;
    let diagnostics = representative_diagnostics();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&diagnostics, ITERATIONS, false));
            optimized.push(measure(&diagnostics, ITERATIONS, true));
        } else {
            optimized.push(measure(&diagnostics, ITERATIONS, true));
            legacy.push(measure(&diagnostics, ITERATIONS, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4} samples={SAMPLE_PAIRS} iterations={ITERATIONS} metrics=9"
    );
    assert!(
        ratio <= 0.50,
        "{MARKER} expected static diagnostic ratio <= 0.50, got {ratio:.4}"
    );
}

fn representative_diagnostics() -> NativeSystemScheduleDiagnostics {
    let mut diagnostics = NativeSystemScheduleDiagnostics::default();
    diagnostics.record_conflicts(5);
    diagnostics.record_main_callback(Duration::from_micros(13), true);
    diagnostics.record_worker_batch(
        &[
            NativeSystemCallbackTiming {
                ready_delay: Duration::from_micros(7),
                callback: Duration::from_micros(23),
            },
            NativeSystemCallbackTiming {
                ready_delay: Duration::from_micros(11),
                callback: Duration::from_micros(31),
            },
        ],
        Duration::from_micros(43),
        2,
        4,
        512,
    );
    diagnostics
}

fn record_legacy(
    diagnostics: &NativeSystemScheduleDiagnostics,
    store: &mut DiagnosticStore,
    frame_index: u64,
) {
    for (path, value, unit) in diagnostics.diagnostic_values() {
        store.record(
            path,
            frame_index,
            value,
            Some(unit),
            ["ecs", "native_system", "schedule"],
        );
    }
}

fn measure(
    diagnostics: &NativeSystemScheduleDiagnostics,
    iterations: usize,
    optimized: bool,
) -> Duration {
    let mut store = DiagnosticStore::new(1);
    let started = Instant::now();
    for frame_index in 0..iterations as u64 {
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
    values[(values.len() - 1) * percentile / 100]
}
