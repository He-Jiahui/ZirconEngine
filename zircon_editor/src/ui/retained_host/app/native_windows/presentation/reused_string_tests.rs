use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use crate::ui::workbench::layout::MainPageId;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ie_editor_native_presentation_reuses_string_buffers() {
    let target = benchmark_target(4 * 1024);
    let bounds = benchmark_bounds(0);
    let mut presentation = seeded_presentation(8 * 1024);
    let expected_allocations = presentation_allocations(&presentation);

    apply_native_floating_presentation_data(&mut presentation, &target, &bounds);

    assert_eq!(
        presentation_allocations(&presentation),
        expected_allocations
    );
    assert!(native_floating_presentation_matches(
        &presentation,
        &target,
        &bounds
    ));
}

#[test]
fn optimization_batch_20260828ie_editor_native_presentation_clones_into_existing_fields() {
    let source = include_str!("../presentation.rs");
    let configure = source
        .split("pub(crate) fn configure_native_floating_window_presentation")
        .nth(1)
        .and_then(|body| {
            body.split("fn apply_native_floating_presentation_data")
                .next()
        })
        .expect("native presentation configuration");
    let apply = source
        .split("fn apply_native_floating_presentation_data")
        .nth(1)
        .and_then(|body| body.split("fn native_floating_presentation_matches").next())
        .expect("in-place native presentation update");

    assert!(configure.contains("apply_native_floating_presentation_data("));
    assert_eq!(apply.matches(".clone_from(&target.window_id.0)").count(), 2);
    assert_eq!(
        apply
            .matches(".clone_from(&target.surface_tree_id.0)")
            .count(),
        2
    );
    assert_eq!(apply.matches(".clone_from(&target.title)").count(), 1);
    assert!(!apply.contains("= target.window_id.0.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ie_editor_reused_native_presentation_strings_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 2 * 1024;
    let target = benchmark_target(8 * 1024);

    let mut warm = seeded_presentation(16 * 1024);
    legacy_apply_native_floating_presentation_data(&mut warm, &target, &benchmark_bounds(0));
    apply_native_floating_presentation_data(&mut warm, &target, &benchmark_bounds(1));
    black_box(warm);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let mut presentation = seeded_presentation(16 * 1024);
            let started = Instant::now();
            for iteration in 0..ITERATIONS {
                legacy_apply_native_floating_presentation_data(
                    black_box(&mut presentation),
                    black_box(&target),
                    black_box(&benchmark_bounds(iteration)),
                );
            }
            black_box(presentation);
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let mut presentation = seeded_presentation(16 * 1024);
            let started = Instant::now();
            for iteration in 0..ITERATIONS {
                apply_native_floating_presentation_data(
                    black_box(&mut presentation),
                    black_box(&target),
                    black_box(&benchmark_bounds(iteration)),
                );
            }
            black_box(presentation);
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
        "EDITOR223_REUSED_NATIVE_PRESENTATION_STRINGS_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_target(bytes: usize) -> NativeFloatingWindowTarget {
    NativeFloatingWindowTarget {
        window_id: MainPageId("window/".repeat(bytes / 7)),
        title: "title/".repeat(bytes / 6),
        bounds: [12.0, 24.0, 640.0, 480.0],
        surface_tree_id: UiTreeId("surface/".repeat(bytes / 8)),
    }
}

fn benchmark_bounds(iteration: usize) -> FrameRect {
    FrameRect {
        x: 12.0 + (iteration % 2) as f32,
        y: 24.0,
        width: 640.0,
        height: 480.0,
    }
}

fn seeded_presentation(capacity: usize) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    seed_string(
        &mut presentation.host_shell.native_floating_window_id,
        capacity,
    );
    seed_string(
        &mut presentation.host_shell.native_surface_tree_id,
        capacity,
    );
    seed_string(&mut presentation.host_shell.native_window_title, capacity);
    seed_string(
        &mut presentation
            .native_floating_surface_data
            .native_floating_window_id,
        capacity,
    );
    seed_string(
        &mut presentation
            .native_floating_surface_data
            .native_surface_tree_id,
        capacity,
    );
    presentation
}

fn seed_string(value: &mut String, capacity: usize) {
    value.reserve(capacity);
    value.push_str("previous");
}

fn presentation_allocations(presentation: &HostWindowPresentationData) -> [*const u8; 5] {
    [
        presentation.host_shell.native_floating_window_id.as_ptr(),
        presentation.host_shell.native_surface_tree_id.as_ptr(),
        presentation.host_shell.native_window_title.as_ptr(),
        presentation
            .native_floating_surface_data
            .native_floating_window_id
            .as_ptr(),
        presentation
            .native_floating_surface_data
            .native_surface_tree_id
            .as_ptr(),
    ]
}

fn legacy_apply_native_floating_presentation_data(
    presentation: &mut HostWindowPresentationData,
    target: &NativeFloatingWindowTarget,
    bounds: &FrameRect,
) {
    presentation.host_shell.native_floating_window_mode = true;
    presentation.host_shell.native_floating_window_id = target.window_id.0.clone();
    presentation.host_shell.native_surface_tree_id = target.surface_tree_id.0.clone();
    presentation.host_shell.native_window_title = target.title.clone();
    presentation.host_shell.native_window_bounds = bounds.clone();
    presentation
        .native_floating_surface_data
        .native_floating_window_id = target.window_id.0.clone();
    presentation
        .native_floating_surface_data
        .native_surface_tree_id = target.surface_tree_id.0.clone();
    presentation
        .native_floating_surface_data
        .native_window_bounds = bounds.clone();
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
