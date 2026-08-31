use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ik_runtime_schema_errors_move_owned_asset_ids() {
    let current_id = benchmark_asset_id(64 * 1024);
    let current_allocation = current_id.as_ptr();
    let v2_id = benchmark_asset_id(64 * 1024);
    let v2_allocation = v2_id.as_ptr();

    let UiAssetError::UnsupportedSchemaVersion { asset_id, .. } =
        unsupported_current_schema_error(current_id, 0)
    else {
        panic!("current schema helper must return the unsupported-version error");
    };
    let UiV2AssetError::UnsupportedSchemaVersion {
        asset_id: v2_asset_id,
        ..
    } = unsupported_v2_schema_error(v2_id, 0)
    else {
        panic!("v2 schema helper must return the unsupported-version error");
    };

    assert_eq!(asset_id.as_ptr(), current_allocation);
    assert_eq!(v2_asset_id.as_ptr(), v2_allocation);
}

#[test]
fn optimization_batch_20260828ik_runtime_version_validation_consumes_documents() {
    let source = include_str!("../document_loader.rs");
    let current = source
        .split("pub(super) fn load_current_ui_document")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn load_ui_v2_document").next())
        .expect("current UI document loader");
    let v2 = source
        .split("fn validate_version")
        .nth(1)
        .and_then(|body| body.split("fn unsupported_current_schema_error").next())
        .expect("v2 version validator");

    assert!(current.contains("unsupported_current_schema_error("));
    assert!(!current.contains("document.asset.id.clone()"));
    assert!(v2.contains("document: UiV2AssetDocument"));
    assert!(v2.contains("unsupported_v2_schema_error("));
    assert!(!v2.contains("document.asset.id.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ik_runtime_owned_ui_schema_error_id_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_v2_schema_error(benchmark_asset_id(64 * 1024), 0));
    black_box(unsupported_v2_schema_error(
        benchmark_asset_id(64 * 1024),
        0,
    ));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_ids = benchmark_asset_ids(ITERATIONS, 64 * 1024);
        let optimized_ids = benchmark_asset_ids(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_ids(legacy_ids, legacy_v2_schema_error));
            optimized_samples.push(measure_ids(optimized_ids, unsupported_v2_schema_error));
        } else {
            optimized_samples.push(measure_ids(optimized_ids, unsupported_v2_schema_error));
            legacy_samples.push(measure_ids(legacy_ids, legacy_v2_schema_error));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME283_OWNED_UI_SCHEMA_ERROR_ID_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
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
    "ui-asset-id/".repeat(bytes / 12)
}

fn benchmark_asset_ids(count: usize, bytes: usize) -> Vec<String> {
    (0..count).map(|_| benchmark_asset_id(bytes)).collect()
}

fn legacy_v2_schema_error(asset_id: String, version: u32) -> UiV2AssetError {
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
