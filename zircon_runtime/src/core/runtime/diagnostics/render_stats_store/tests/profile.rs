use std::sync::Arc;

use crate::core::framework::render::{
    RenderBudgetKey, RenderFrameProfile, RenderPassNativeResourceCreateMetrics,
    RenderPassProfileEntry, RenderStats, RenderSubsystemProfileEntry,
};
use crate::core::runtime::diagnostics::DiagnosticStore;

use super::record;

#[test]
fn profile_diagnostics_mirror_resolved_gpu_budget_data_without_dynamic_pass_paths() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_frame_profile: Arc::new(RenderFrameProfile {
            cpu_submit_time_us: 1_500,
            gpu_frame_time_us: Some(4_000),
            parallel_recording_eligible_stage_count: 1,
            parallel_recording_eligible_bucket_count: 3,
            parallel_recording_executed_stage_count: 1,
            parallel_recording_executed_bucket_count: 2,
            profile_latency_frames: 2,
            budget_warning_count: 1,
            passes: vec![
                RenderPassProfileEntry {
                    native_resource_creates: RenderPassNativeResourceCreateMetrics::new(
                        1, 2, 3, 4, 5, 6, 7,
                    ),
                    ..RenderPassProfileEntry::default()
                },
                RenderPassProfileEntry {
                    native_resource_creates: RenderPassNativeResourceCreateMetrics::new(
                        10, 20, 30, 40, 50, 60, 70,
                    ),
                    ..RenderPassProfileEntry::default()
                },
            ],
            subsystems: vec![RenderSubsystemProfileEntry {
                key: RenderBudgetKey::BasePass,
                gpu_time_us: Some(4_000),
                budget_us: 3_200,
                over_budget: true,
            }],
            ..RenderFrameProfile::default()
        }),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.profile.cpu_submit_time_us",
        1_500.0,
        "microseconds",
    );
    assert_series(
        &store,
        "render.profile.gpu_frame_time_us",
        4_000.0,
        "microseconds",
    );
    assert_series(
        &store,
        "render.profile.parallel_recording.eligible_stage_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.profile.parallel_recording.eligible_bucket_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.profile.parallel_recording.executed_stage_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.profile.parallel_recording.executed_bucket_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.profile.subsystem.base_pass.gpu_time_us",
        4_000.0,
        "microseconds",
    );
    assert_series(
        &store,
        "render.profile.subsystem.base_pass.over_budget",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.profile.native_resource_create.total_count",
        308.0,
        "count",
    );
    assert_series(
        &store,
        "render.profile.native_resource_create.buffer_count",
        11.0,
        "count",
    );
    assert_series(
        &store,
        "render.profile.native_resource_create.render_pipeline_count",
        77.0,
        "count",
    );
    assert!(store
        .snapshot()
        .series
        .iter()
        .all(|series| !series.path.as_str().contains("opaque")));
}

fn assert_series(store: &DiagnosticStore, path: &str, value: f64, unit: &str) {
    let series = store
        .snapshot()
        .series
        .into_iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
    assert_eq!(series.current, Some(value));
    assert_eq!(series.unit.as_deref(), Some(unit));
}
