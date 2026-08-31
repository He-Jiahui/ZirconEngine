use std::hint::black_box;
use std::time::Instant;

use crate::asset::AssetUri;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ht_runtime_preserves_manifest_toml_and_round_trip() {
    let mut manifest = benchmark_manifest(8, 64);
    manifest.format_version = 1;

    let optimized = serialize_current_project_manifest(&manifest).unwrap();
    let legacy = legacy_serialize_current_project_manifest(&manifest).unwrap();

    assert_eq!(optimized, legacy);
    let decoded = ProjectManifest::from_toml_str(&optimized).unwrap().value;
    let mut expected = manifest;
    expected.format_version = PROJECT_MANIFEST_FORMAT_VERSION;
    assert_eq!(decoded, expected);
}

#[test]
fn optimization_batch_20260828ht_runtime_save_borrows_project_manifest_fields() {
    let source = include_str!("../save.rs");
    let save = source
        .split("pub(crate) fn save_with_atomic_fault")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("project manifest save implementation");

    assert!(source.contains("struct CurrentProjectManifest<'a>"));
    assert!(source.contains("impl Serialize for CurrentProjectManifest<'_>"));
    assert!(save.contains("serialize_current_project_manifest(self)"));
    assert!(!save.contains("let mut current = self.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ht_runtime_borrowed_project_manifest_save_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let manifest = benchmark_manifest(512, 4 * 1024);

    black_box(serialize_current_project_manifest(&manifest).unwrap());
    black_box(legacy_serialize_current_project_manifest(&manifest).unwrap());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_serialize_current_project_manifest(black_box(&manifest)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(serialize_current_project_manifest(black_box(&manifest)).unwrap());
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
        "RUNTIME266_BORROWED_PROJECT_MANIFEST_SAVE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_serialize_current_project_manifest(
    manifest: &ProjectManifest,
) -> Result<String, toml::ser::Error> {
    let mut current = manifest.clone();
    current.format_version = PROJECT_MANIFEST_FORMAT_VERSION;
    toml::to_string_pretty(&current)
}

fn benchmark_manifest(uri_count: usize, uri_bytes: usize) -> ProjectManifest {
    let suffix = "x".repeat(uri_bytes);
    let mut manifest = ProjectManifest::new(
        "benchmark-project",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.ui_roots = (0..uri_count)
        .map(|index| AssetUri::parse(&format!("res://ui/{index}/{suffix}.zui")).unwrap())
        .collect();
    manifest.asset_manifest = Some(format!("asset-manifest-{suffix}.json"));
    manifest
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
