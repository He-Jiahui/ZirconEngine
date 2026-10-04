use std::hint::black_box;
use std::time::Instant;

use super::{parse_shader_ide_wgsl_module, validate_shader_ide_wgsl_module};

#[derive(Clone)]
struct BenchmarkEntryPoint {
    name: String,
    stage_metadata: [u32; 8],
}

#[test]
fn shader_ide_wgsl_parse_reports_module_id() {
    let error = parse_shader_ide_wgsl_module("project::broken", "fn broken( {")
        .expect_err("invalid WGSL should fail");

    assert!(error.to_string().contains("project::broken"), "{error}");
}

#[test]
fn shader_ide_wgsl_validation_reports_entry_points() {
    let validation = validate_shader_ide_wgsl_module(
        "project::preview",
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(vertex_index);
    return vec4<f32>(x, 0.0, 0.0, 1.0);
}
"#,
    )
    .expect("valid preview WGSL");

    assert_eq!(validation.entry_points, vec!["vs_main".to_string()]);
}

#[test]
fn optimization_batch_hk_runtime587_shader_entry_points_move_names() {
    let source = include_str!("../ide_validation.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let extraction = production
        .split("fn shader_entry_points")
        .nth(1)
        .expect("shader entry point extraction");

    assert!(production.contains("shader_entry_points(module)"));
    assert!(extraction.contains(".entry_points\n        .into_iter()"));
    assert!(!extraction.contains("entry_point.name.clone()"));
}

fn benchmark_entry_points(count: usize) -> Vec<BenchmarkEntryPoint> {
    (0..count)
        .map(|index| BenchmarkEntryPoint {
            name: format!(
                "runtime587_shader_entry_point_{index:05}_{}",
                "stable-project-qualified-name".repeat(3)
            ),
            stage_metadata: [index as u32; 8],
        })
        .collect()
}

fn legacy_entry_point_names(entry_points: Vec<BenchmarkEntryPoint>) -> Vec<String> {
    entry_points
        .iter()
        .map(|entry_point| {
            black_box(entry_point.stage_metadata);
            entry_point.name.clone()
        })
        .collect()
}

fn moved_entry_point_names(entry_points: Vec<BenchmarkEntryPoint>) -> Vec<String> {
    entry_points
        .into_iter()
        .map(|entry_point| {
            black_box(entry_point.stage_metadata);
            entry_point.name
        })
        .collect()
}

fn measure_entry_point_projection(
    input: Vec<BenchmarkEntryPoint>,
    project: fn(Vec<BenchmarkEntryPoint>) -> Vec<String>,
) -> u128 {
    let started = Instant::now();
    let names = project(input);
    black_box(&names);
    drop(names);
    started.elapsed().as_nanos().max(1)
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hk_runtime587_shader_entry_point_move_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const ENTRY_POINTS_PER_SAMPLE: usize = 16_384;

    let seed = benchmark_entry_points(ENTRY_POINTS_PER_SAMPLE);
    let expected = legacy_entry_point_names(seed.clone());
    let optimized = moved_entry_point_names(seed.clone());
    assert_eq!(optimized, expected);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_input = seed.clone();
        let optimized_input = seed.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure_entry_point_projection(
                legacy_input,
                legacy_entry_point_names,
            ));
            optimized_samples.push(measure_entry_point_projection(
                optimized_input,
                moved_entry_point_names,
            ));
        } else {
            optimized_samples.push(measure_entry_point_projection(
                optimized_input,
                moved_entry_point_names,
            ));
            legacy_samples.push(measure_entry_point_projection(
                legacy_input,
                legacy_entry_point_names,
            ));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME587_SHADER_ENTRY_POINT_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             entry_points_per_sample={ENTRY_POINTS_PER_SAMPLE} \
             legacy_name_clones_per_sample={ENTRY_POINTS_PER_SAMPLE} \
             optimized_name_clones_per_sample=0 legacy_p95_ns={legacy_p95} \
             optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 45,
        "moved entry point P95 {optimized_p95}ns exceeded 45% of cloned P95 {legacy_p95}ns"
    );
}
