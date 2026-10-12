use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::relative_uri_path;

const SAMPLE_PAIRS: usize = 21;
const PATHS_PER_SAMPLE: usize = 16_384;
const COMPONENTS_PER_PATH: usize = 32;

#[test]
fn optimization_batch_20260826dm_runtime156_project_source_uri_path_preserves_components() {
    let path = PathBuf::from("models")
        .join("characters")
        .join("hero")
        .join("body.mesh");
    assert_eq!(relative_uri_path(&path), "models/characters/hero/body.mesh");
    assert_eq!(relative_uri_path(Path::new("")), "");
}

#[test]
fn optimization_batch_20260826dm_runtime156_project_source_uri_path_shares_direct_join() {
    let path = fixture_path();
    let relative = relative_uri_path(&path);
    assert_eq!(relative.len(), relative.capacity());

    let source = include_str!("../source_uri_for_path.rs");
    assert_eq!(source.matches("relative_uri_path(").count(), 3);
    assert!(source.contains("String::with_capacity(path.as_os_str().len())"));
    assert!(source.contains("relative.push_str("));
    assert!(!source.contains("collect::<Vec<_>>()\n            .join(\"/\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dm_runtime156_project_source_uri_path_direct_join_bench() {
    let path = fixture_path();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&path, legacy_relative_uri_path));
            optimized_samples.push(measure(&path, relative_uri_path));
        } else {
            optimized_samples.push(measure(&path, relative_uri_path));
            legacy_samples.push(measure(&path, legacy_relative_uri_path));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME156_PROJECT_SOURCE_URI_PATH_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
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
        "direct source URI path joining P95 {optimized_p95_ns}ns must be at most 70% of component-vector joining P95 {legacy_p95_ns}ns"
    );
}

fn fixture_path() -> PathBuf {
    let mut path = PathBuf::new();
    for index in 0..COMPONENTS_PER_PATH {
        path.push(format!("asset_{index:02}"));
    }
    path
}

fn legacy_relative_uri_path(path: &Path) -> String {
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
