use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hy_editor_patch_moves_literal_into_final_state_value() {
    let literal = "x".repeat(4 * 1024);
    let allocation = literal.as_ptr();

    let patch = asset_editor_projection_patch("Control", "widget.text", literal);

    let UiValue::String(attribute) = patch.attributes.get(VALUE_PROPERTY).unwrap() else {
        panic!("expected string attribute");
    };
    let UiValue::String(state_value) = patch.state_values.get("widget.text").unwrap() else {
        panic!("expected string state value");
    };
    assert_eq!(attribute, state_value);
    assert_ne!(attribute.as_ptr(), allocation);
    assert_eq!(state_value.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828hy_editor_asset_mutation_borrows_literal_before_patch_move() {
    let source = include_str!("../asset_editor.rs");
    let apply = source
        .split("pub(crate) fn apply_asset_editor_component_envelope")
        .nth(1)
        .and_then(|body| body.split("fn asset_editor_projection_patch").next())
        .expect("asset editor component adapter implementation");

    assert!(apply.contains("asset_editor_projection_patch("));
    assert!(apply.contains("&literal"));
    assert!(!apply.contains("literal.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hy_editor_borrowed_asset_literal_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 128;
    let literal = "x".repeat(64 * 1024);

    black_box(legacy_asset_editor_projection_patch(
        "Control",
        "widget.text",
        literal.clone(),
    ));
    black_box(asset_editor_projection_patch(
        "Control",
        "widget.text",
        literal.clone(),
    ));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS).map(|_| literal.clone()).collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS).map(|_| literal.clone()).collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_projection(legacy_inputs, |literal| {
                legacy_asset_editor_projection_patch("Control", "widget.text", literal)
            }));
            optimized_samples.push(measure_projection(optimized_inputs, |literal| {
                black_box(literal.as_str());
                asset_editor_projection_patch("Control", "widget.text", literal)
            }));
        } else {
            optimized_samples.push(measure_projection(optimized_inputs, |literal| {
                black_box(literal.as_str());
                asset_editor_projection_patch("Control", "widget.text", literal)
            }));
            legacy_samples.push(measure_projection(legacy_inputs, |literal| {
                legacy_asset_editor_projection_patch("Control", "widget.text", literal)
            }));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR217_BORROWED_ASSET_EDITOR_LITERAL_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_asset_editor_projection_patch(
    control_id: &str,
    path: &str,
    literal: String,
) -> UiComponentProjectionPatch {
    black_box(literal.clone());
    UiComponentProjectionPatch::new(control_id)
        .with_attribute(VALUE_PROPERTY, UiValue::String(literal.clone()))
        .with_state_value(path, UiValue::String(literal.clone()))
}

fn measure_projection(
    inputs: Vec<String>,
    mut project: impl FnMut(String) -> UiComponentProjectionPatch,
) -> u128 {
    let started = Instant::now();
    for literal in inputs {
        black_box(project(black_box(literal)));
    }
    started.elapsed().as_nanos()
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
