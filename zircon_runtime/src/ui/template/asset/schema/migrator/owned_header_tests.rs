use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::template::UiAssetKind;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828im_runtime_migrator_moves_owned_header_id() {
    let header = benchmark_header(64 * 1024, u32::MAX);
    let allocation = header.id.as_ptr();

    let UiAssetError::UnsupportedSchemaVersion { asset_id, .. } =
        validate_owned_source_header(header).unwrap_err()
    else {
        panic!("owned header validator must return unsupported-version error");
    };

    assert_eq!(asset_id.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828im_runtime_migrator_uses_consuming_header_validation() {
    let source = include_str!("../migrator.rs");
    let entry = source
        .split("pub fn migrate_toml_str")
        .nth(1)
        .and_then(|body| body.split("fn migrate_tree_asset").next())
        .expect("schema migrator string entry");
    let owned = source
        .split("fn validate_owned_source_header")
        .nth(1)
        .and_then(|body| body.split("fn push_version_bump_step").next())
        .expect("owned source header validator");

    assert!(entry.contains("validate_owned_source_header(parse_asset_header_value(&value)?)"));
    assert!(!entry.contains("reject_unsupported_source_version(&header)"));
    assert!(owned.contains("asset_id: header.id"));
    assert!(!owned.contains("header.id.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828im_runtime_owned_migrator_schema_header_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_validate_header(benchmark_header(
        64 * 1024,
        u32::MAX,
    )));
    black_box(validate_owned_source_header(benchmark_header(
        64 * 1024,
        u32::MAX,
    )));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_headers = benchmark_headers(ITERATIONS, 64 * 1024);
        let optimized_headers = benchmark_headers(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_headers(legacy_headers, legacy_validate_header));
            optimized_samples.push(measure_headers(
                optimized_headers,
                validate_owned_source_header,
            ));
        } else {
            optimized_samples.push(measure_headers(
                optimized_headers,
                validate_owned_source_header,
            ));
            legacy_samples.push(measure_headers(legacy_headers, legacy_validate_header));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME285_OWNED_MIGRATOR_SCHEMA_HEADER_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_header(bytes: usize, version: u32) -> UiAssetHeader {
    UiAssetHeader {
        kind: UiAssetKind::Layout,
        id: "migrator-asset-id/".repeat(bytes / 18),
        version,
        display_name: String::new(),
    }
}

fn benchmark_headers(count: usize, bytes: usize) -> Vec<UiAssetHeader> {
    (0..count)
        .map(|_| benchmark_header(bytes, u32::MAX))
        .collect()
}

fn legacy_validate_header(header: UiAssetHeader) -> Result<UiAssetHeader, UiAssetError> {
    if !UiAssetSchemaVersionPolicy::is_supported_source_schema(header.version) {
        return Err(UiAssetError::UnsupportedSchemaVersion {
            asset_id: header.id.clone(),
            version: header.version,
            current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        });
    }
    Ok(header)
}

fn measure_headers(
    headers: Vec<UiAssetHeader>,
    mut validate: impl FnMut(UiAssetHeader) -> Result<UiAssetHeader, UiAssetError>,
) -> u128 {
    let started = Instant::now();
    for header in headers {
        black_box(validate(black_box(header)));
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
