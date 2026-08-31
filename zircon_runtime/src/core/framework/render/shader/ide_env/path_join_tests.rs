use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::shader_ide_relative_path_string;

const SAMPLE_PAIRS: usize = 21;
const PATHS_PER_SAMPLE: usize = 16_384;
const COMPONENTS_PER_PATH: usize = 32;

#[test]
fn optimization_batch_20260826dk_runtime154_shader_ide_path_preserves_component_contract() {
    let path = PathBuf::from("modules")
        .join("project")
        .join("surface")
        .join("common.wgsl");
    assert_eq!(
        shader_ide_relative_path_string(&path),
        "modules/project/surface/common.wgsl"
    );
    assert_eq!(shader_ide_relative_path_string(Path::new("")), "");
}

#[test]
fn optimization_batch_20260826dk_runtime154_shader_ide_path_appends_without_component_vec() {
    let source = include_str!("../ide_env.rs");
    let function_start = source
        .find("pub fn shader_ide_relative_path_string")
        .expect("path formatter should remain present");
    let function_tail = &source[function_start..];
    let function_end = function_tail
        .find("\n}\n")
        .expect("path formatter should remain bounded");
    let function = &function_tail[..function_end];

    assert!(function.contains("String::with_capacity(path.as_os_str().len())"));
    assert!(function.contains("relative_path.push_str("));
    assert!(!function.contains("collect::<Vec<_>>()"));
    assert!(!function.contains(".join(\"/\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dk_runtime154_shader_ide_path_direct_join_bench() {
    let path = fixture_path();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&path, legacy_relative_path_string));
            optimized_samples.push(measure(&path, shader_ide_relative_path_string));
        } else {
            optimized_samples.push(measure(&path, shader_ide_relative_path_string));
            legacy_samples.push(measure(&path, legacy_relative_path_string));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME154_SHADER_IDE_PATH_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
paths_per_sample={PATHS_PER_SAMPLE} components_per_path={COMPONENTS_PER_PATH} \
legacy_temporary_vecs_per_sample={PATHS_PER_SAMPLE} optimized_temporary_vecs_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct shader IDE path join P95 {optimized_p95_ns}ns must be at most 70% of component-vector joining P95 {legacy_p95_ns}ns"
    );
}

fn fixture_path() -> PathBuf {
    let mut path = PathBuf::new();
    for index in 0..COMPONENTS_PER_PATH {
        path.push(format!("module_{index:02}"));
    }
    path
}

fn legacy_relative_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn measure(path: &Path, render: fn(&Path) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..PATHS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(path))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
