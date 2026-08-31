use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828il_runtime_public_v2_error_moves_asset_id() {
    let asset_id = benchmark_asset_id(64 * 1024);
    let allocation = asset_id.as_ptr();

    let UiV2AssetError::UnsupportedSchemaVersion { asset_id, .. } =
        unsupported_schema_error(asset_id, 0)
    else {
        panic!("schema helper must return the unsupported-version error");
    };

    assert_eq!(asset_id.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828il_runtime_public_v2_validation_consumes_document() {
    let source = include_str!("../loader.rs");
    let load = source
        .split("pub fn load_toml_str")
        .nth(1)
        .and_then(|body| body.split("pub fn load_toml_file").next())
        .expect("public v2 string loader");
    let validate = source
        .split("fn validate_version")
        .nth(1)
        .and_then(|body| body.split("fn unsupported_schema_error").next())
        .expect("public v2 version validator");

    assert!(load.contains("validate_version(document)"));
    assert!(validate.contains("document: UiV2AssetDocument"));
    assert!(validate.contains("unsupported_schema_error("));
    assert!(!validate.contains("document.asset.id.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828il_runtime_owned_public_v2_schema_error_id_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_schema_error(benchmark_asset_id(64 * 1024), 0));
    black_box(unsupported_schema_error(benchmark_asset_id(64 * 1024), 0));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_ids = benchmark_asset_ids(ITERATIONS, 64 * 1024);
        let optimized_ids = benchmark_asset_ids(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_ids(legacy_ids, legacy_schema_error));
            optimized_samples.push(measure_ids(optimized_ids, unsupported_schema_error));
        } else {
            optimized_samples.push(measure_ids(optimized_ids, unsupported_schema_error));
            legacy_samples.push(measure_ids(legacy_ids, legacy_schema_error));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME284_OWNED_PUBLIC_V2_SCHEMA_ERROR_ID_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_asset_id(bytes: usize) -> String {
    "public-ui-v2-id/".repeat(bytes / 16)
}

fn benchmark_asset_ids(count: usize, bytes: usize) -> Vec<String> {
    (0..count).map(|_| benchmark_asset_id(bytes)).collect()
}

fn legacy_schema_error(asset_id: String, version: u32) -> UiV2AssetError {
    UiV2AssetError::UnsupportedSchemaVersion {
        asset_id: asset_id.clone(),
        version,
        expected: UI_V2_ASSET_SCHEMA_VERSION,
    }
}

fn measure_ids(
    asset_ids: Vec<String>,
    mut convert: impl FnMut(String, u32) -> UiV2AssetError,
) -> u128 {
    let started = Instant::now();
    for asset_id in asset_ids {
        black_box(convert(black_box(asset_id), 0));
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
