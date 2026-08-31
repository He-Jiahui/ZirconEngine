use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{remove_failed_lifecycle_stage, EditorPluginLifecycleStage};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 20_000;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn stage_fixture() -> Vec<EditorPluginLifecycleStage> {
    vec![
        EditorPluginLifecycleStage::Loaded,
        EditorPluginLifecycleStage::Enabled,
        EditorPluginLifecycleStage::Disabled,
        EditorPluginLifecycleStage::Unloaded,
        EditorPluginLifecycleStage::HotReloaded,
        EditorPluginLifecycleStage::EnteredPlayMode,
        EditorPluginLifecycleStage::ExitedPlayMode,
        EditorPluginLifecycleStage::SceneChanged,
        EditorPluginLifecycleStage::AssetChanged,
        EditorPluginLifecycleStage::UiMessage,
    ]
}

fn legacy_remove_failed_lifecycle_stage(
    failed_stages: &mut Vec<EditorPluginLifecycleStage>,
    stage: &EditorPluginLifecycleStage,
) -> bool {
    let before = failed_stages.len();
    failed_stages.retain(|failed_stage| failed_stage != stage);
    failed_stages.len() != before
}

#[test]
fn editor06_descriptor_lifecycle_stage_membership() {
    let mut failed_stages = stage_fixture();
    assert!(remove_failed_lifecycle_stage(
        &mut failed_stages,
        &EditorPluginLifecycleStage::Loaded
    ));
    assert!(!failed_stages.contains(&EditorPluginLifecycleStage::Loaded));
    assert_eq!(failed_stages.len(), 9);
    assert!(!remove_failed_lifecycle_stage(
        &mut failed_stages,
        &EditorPluginLifecycleStage::Loaded
    ));
}

#[test]
fn editor06_descriptor_lifecycle_stage_source_contract() {
    let source = include_str!("../registration.rs");
    assert!(source.contains("failed_stages.swap_remove(index)"));
    assert!(!source.contains("failed_lifecycle_stages\n                .retain"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_descriptor_lifecycle_stage_bench() {
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut stages = stage_fixture();
                black_box(legacy_remove_failed_lifecycle_stage(
                    &mut stages,
                    &EditorPluginLifecycleStage::Loaded,
                ));
                black_box(stages);
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut stages = stage_fixture();
                black_box(remove_failed_lifecycle_stage(
                    &mut stages,
                    &EditorPluginLifecycleStage::Loaded,
                ));
                black_box(stages);
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR06_FAILED_LIFECYCLE_STAGE_SWAP_REMOVE_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} failed_stage_entries={} retain_moves=9->swap_remove_moves=1",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        stage_fixture().len(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
