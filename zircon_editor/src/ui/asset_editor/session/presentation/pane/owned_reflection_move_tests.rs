use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute,
};
use zircon_runtime_interface::ui::template::UiAssetKind;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hv_editor_moves_owned_reflection_allocations() {
    let mut reflection = benchmark_reflection(32, 256);
    let asset_id = reflection.route.asset_id.as_ptr();
    let conflict_summary = reflection.external_conflict_summary.as_ptr();
    let stale_import_items = reflection.stale_import_items.as_ptr();
    let emergency_summary = reflection.emergency_summary.as_ptr();
    let style_class_items = reflection.style_inspector.classes.as_ptr();
    let last_error = reflection.last_error.as_ref().expect("last error").as_ptr();

    let owned = take_owned_pane_reflection_fields(&mut reflection);

    assert_eq!(owned.asset_id.as_ptr(), asset_id);
    assert_eq!(owned.external_conflict_summary.as_ptr(), conflict_summary);
    assert_eq!(owned.stale_import_items.as_ptr(), stale_import_items);
    assert_eq!(owned.emergency_summary.as_ptr(), emergency_summary);
    assert_eq!(owned.style_class_items.as_ptr(), style_class_items);
    assert_eq!(owned.last_error.as_ptr(), last_error);
}

#[test]
fn optimization_batch_20260828hv_editor_pane_projection_consumes_reflection_fields() {
    let source = include_str!("../pane.rs");
    let pane_presentation = source
        .split("pub fn pane_presentation")
        .nth(1)
        .expect("pane presentation implementation");

    assert!(pane_presentation.contains("take_owned_pane_reflection_fields(&mut reflection)"));
    assert!(!pane_presentation.contains("reflection.route.asset_id.clone()"));
    assert!(!pane_presentation.contains("reflection.external_conflict_summary.clone()"));
    assert!(!pane_presentation.contains("reflection.stale_import_items.clone()"));
    assert!(!pane_presentation.contains("reflection.style_inspector.classes.clone()"));
    assert!(!pane_presentation.contains("reflection.last_error.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hv_editor_owned_reflection_fields_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 32;
    let reflection = benchmark_reflection(512, 4 * 1024);

    black_box(legacy_clone_pane_reflection_fields(&reflection));
    let mut warmup = reflection.clone();
    black_box(take_owned_pane_reflection_fields(&mut warmup));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let mut optimized_inputs = (0..ITERATIONS)
            .map(|_| reflection.clone())
            .collect::<Vec<_>>();
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_clone_pane_reflection_fields(black_box(&reflection)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for input in &mut optimized_inputs {
                black_box(take_owned_pane_reflection_fields(black_box(input)));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR214_OWNED_PANE_REFLECTION_MOVE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_reflection(item_count: usize, item_bytes: usize) -> UiAssetEditorReflectionModel {
    let payload = "x".repeat(item_bytes);
    let route = UiAssetEditorRoute::new(
        format!("editor.test.{payload}"),
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut reflection = UiAssetEditorReflectionModel::new(route, "Benchmark");
    reflection.external_conflict_summary = format!("conflict-{payload}");
    reflection.stale_import_items = (0..item_count)
        .map(|index| format!("stale-{index}-{payload}"))
        .collect();
    reflection.emergency_summary = format!("emergency-{payload}");
    reflection.style_inspector.classes = (0..item_count)
        .map(|index| format!("class-{index}-{payload}"))
        .collect();
    reflection.last_error = Some(format!("error-{payload}"));
    reflection
}

fn legacy_clone_pane_reflection_fields(
    reflection: &UiAssetEditorReflectionModel,
) -> OwnedPaneReflectionFields {
    OwnedPaneReflectionFields {
        asset_id: reflection.route.asset_id.clone(),
        external_conflict_summary: reflection.external_conflict_summary.clone(),
        stale_import_items: reflection.stale_import_items.clone(),
        emergency_summary: reflection.emergency_summary.clone(),
        style_class_items: reflection.style_inspector.classes.clone(),
        last_error: reflection.last_error.clone().unwrap_or_default(),
    }
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
