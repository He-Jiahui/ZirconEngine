use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};

use super::{ProjectGenerationObservation, ProjectGenerationPhase};

#[test]
fn typed_observation_publishes_generation_work_counters() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "project-generation-observation".to_owned();
    config.max_spans = 16;
    config.max_counters = 64;
    start_capture(config);

    let mut observation = ProjectGenerationObservation::new();
    observation.record_sources(3, 4);
    observation.record_metadata_inventory(3, 2);
    observation.record_metadata_deserialize(41);
    observation.record_metadata_deserialize(59);
    observation.record_source_bytes(1_024);
    observation.record_restored_source();
    observation.record_imported_source();
    observation.record_failed_source();
    observation.record_artifact(4_096, 2_048, 2, 256);
    observation.record_changed_metadata(2);
    observation.record_prepared_writes(4, 3_072);
    observation.mark_prepare_succeeded();
    observation.mark_commit_succeeded();
    drop(observation);

    let snapshot = snapshot();
    reset_capture();
    assert_counter(&snapshot, "asset.project_generation.source_count", 3.0);
    assert_counter(
        &snapshot,
        "asset.project_generation.compound_member_path_count",
        4.0,
    );
    assert_counter(
        &snapshot,
        "asset.project_generation.metadata_deserialize_count",
        2.0,
    );
    assert_counter(
        &snapshot,
        "asset.project_generation.metadata_deserialize_bytes",
        100.0,
    );
    assert_counter(
        &snapshot,
        "asset.project_generation.artifact_compressed_bytes",
        2_048.0,
    );
    assert_counter(
        &snapshot,
        "asset.project_generation.committed_write_count",
        4.0,
    );
    assert_counter(&snapshot, "asset.project_generation.commit_succeeded", 1.0);
}

#[test]
fn typed_phase_uses_stable_profiler_path() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "project-generation-phase".to_owned();
    config.max_spans = 4;
    start_capture(config);

    {
        let _phase = ProjectGenerationPhase::MetadataProjection.enter();
    }

    let snapshot = snapshot();
    reset_capture();
    assert!(snapshot
        .spans
        .iter()
        .any(|span| { span.path == "asset/project_generation.phase:metadata_projection" }));
}

fn assert_counter(
    snapshot: &crate::core::runtime::diagnostics::profiling::ProfileSnapshot,
    name: &str,
    expected: f64,
) {
    assert_eq!(
        snapshot
            .counters
            .iter()
            .find(|counter| counter.name == name)
            .map(|counter| counter.value),
        Some(expected),
        "missing or incorrect counter {name}"
    );
}
