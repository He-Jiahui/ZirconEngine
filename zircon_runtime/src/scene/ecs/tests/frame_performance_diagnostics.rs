use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::diagnostics::{DiagnosticStore, FrameDiagnostics};
use crate::core::CoreRuntime;

use super::EcsFramePerformanceDiagnostics;

#[test]
fn ecs_frame_performance_diagnostics_uses_scene_ecs_frame_domain() {
    let diagnostics = EcsFramePerformanceDiagnostics::default();
    let status = diagnostics.frame_diagnostics_status();

    assert_eq!(status.domain, "scene.ecs");
    assert!(status.available);
    assert_eq!(status.error, None);
}

#[test]
fn optimization_wave_20260824p_runtime03_ecs_publish_matches_direct_store_recording() {
    let diagnostics = EcsFramePerformanceDiagnostics::default();
    let mut expected = DiagnosticStore::default();
    diagnostics.record_diagnostics(&mut expected, 41);

    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    diagnostics.publish(&core, 41);

    let actual = core.diagnostic_store_snapshot();
    assert_eq!(actual, expected.snapshot());
    assert_eq!(actual.series.len(), 60);
}

#[test]
fn ecs_frame_performance_diagnostics_exports_propagation_visit_and_write_counts() {
    let mut diagnostics = EcsFramePerformanceDiagnostics::default();
    diagnostics.derived_state.active_propagation_entities = 7;
    diagnostics
        .derived_state
        .active_propagation_written_entities = 3;
    diagnostics.derived_state.world_matrix_propagation_entities = 11;
    diagnostics
        .derived_state
        .world_matrix_propagation_written_entities = 5;

    let mut store = DiagnosticStore::default();
    diagnostics.record_diagnostics(&mut store, 23);
    let snapshot = store.snapshot();
    let value = |path: &str| {
        snapshot
            .series
            .iter()
            .find(|series| series.path.as_str() == path)
            .and_then(|series| series.current)
            .unwrap_or_else(|| panic!("missing diagnostic {path}"))
    };

    assert_eq!(
        value(super::ECS_DERIVED_STATE_ACTIVE_PROPAGATION_ENTITIES_DIAGNOSTIC),
        7.0
    );
    assert_eq!(
        value(super::ECS_DERIVED_STATE_ACTIVE_PROPAGATION_WRITTEN_ENTITIES_DIAGNOSTIC),
        3.0
    );
    assert_eq!(
        value(super::ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_ENTITIES_DIAGNOSTIC),
        11.0
    );
    assert_eq!(
        value(super::ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_WRITTEN_ENTITIES_DIAGNOSTIC),
        5.0
    );
}

#[test]
fn optimization_wave_20260824p_runtime03_ecs_publish_uses_one_diagnostic_store_update() {
    let source = include_str!("../frame_performance_diagnostics.rs");
    let publish = source
        .split("pub fn publish(&self, core: &CoreHandle, frame_index: u64)")
        .nth(1)
        .and_then(|source| source.split("impl FrameDiagnostics").next())
        .expect("ECS diagnostics publish implementation");

    assert_eq!(publish.matches("update_diagnostic_store").count(), 1);
    assert!(!publish.contains(".record_diagnostic("));
    assert!(!publish.contains("bundle_transactions.publish"));
    assert!(!publish.contains("native_system_schedule.publish"));
}

#[test]
#[ignore = "managed Runtime03 performance evidence"]
fn optimization_wave_20260824p_runtime03_ecs_diagnostic_batch_publish_evidence() {
    const SERIES_PER_PUBLISH: u64 = 60;
    const PUBLISHES: u64 = 25_000;
    const MAX_ELAPSED: Duration = Duration::from_secs(3);

    let diagnostics = EcsFramePerformanceDiagnostics::default();
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let started = Instant::now();
    for frame_index in 0..PUBLISHES {
        black_box(&diagnostics).publish(black_box(&core), frame_index);
    }
    let elapsed = started.elapsed();

    assert!(elapsed <= MAX_ELAPSED);
    let lock_acquisitions_before = SERIES_PER_PUBLISH * PUBLISHES;
    let lock_acquisitions_after = PUBLISHES;
    let lock_reduction_percent =
        (1.0 - lock_acquisitions_after as f64 / lock_acquisitions_before as f64) * 100.0;
    let publishes_per_second = PUBLISHES as f64 / elapsed.as_secs_f64();
    println!(
        "RUNTIME_DIAGNOSTICS_BENCH_V1 kind=ecs_batch_publish publishes={} series_per_publish={} lock_acquisitions_before={} lock_acquisitions_after={} lock_reduction_percent={:.4} elapsed_ns={} target_ns={} publishes_per_second={:.2}",
        PUBLISHES,
        SERIES_PER_PUBLISH,
        lock_acquisitions_before,
        lock_acquisitions_after,
        lock_reduction_percent,
        elapsed.as_nanos(),
        MAX_ELAPSED.as_nanos(),
        publishes_per_second,
    );
}
