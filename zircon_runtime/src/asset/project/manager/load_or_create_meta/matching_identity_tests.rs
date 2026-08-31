use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828in_runtime_matching_meta_identity_keeps_locator_allocations() {
    let uri = benchmark_uri(64 * 1024, "stable-label");
    let mut meta = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Data);
    let path_allocation = meta.url.path().as_ptr();
    let label_allocation = meta.url.label().expect("benchmark label").as_ptr();

    refresh_loaded_meta_identity(&mut meta, &uri, AssetKind::Texture);

    assert_eq!(meta.url.path().as_ptr(), path_allocation);
    assert_eq!(
        meta.url.label().expect("retained benchmark label").as_ptr(),
        label_allocation
    );
    assert_eq!(meta.asset_kind, AssetKind::Texture);

    let replacement = benchmark_uri(64 * 1024, "replacement-label");
    refresh_loaded_meta_identity(&mut meta, &replacement, AssetKind::Mesh);
    assert_eq!(meta.url, replacement);
    assert_eq!(meta.asset_kind, AssetKind::Mesh);
}

#[test]
fn optimization_batch_20260828in_runtime_loaded_meta_uses_matching_identity_guard() {
    let source = include_str!("../load_or_create_meta.rs");
    let loaded = source
        .split("if meta_path.exists()")
        .nth(1)
        .and_then(|body| body.split("Ok(mint_meta").next())
        .expect("loaded meta path");
    let refresh = source
        .split("fn refresh_loaded_meta_identity")
        .nth(1)
        .and_then(|body| body.split("fn mint_meta").next())
        .expect("loaded identity refresh helper");

    assert!(loaded.contains("refresh_loaded_meta_identity(&mut meta, uri, kind)"));
    assert!(!loaded.contains("meta.url = uri.clone()"));
    assert!(refresh.contains("if &meta.url != uri"));
    assert!(refresh.contains("meta.url = uri.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828in_runtime_matching_meta_identity_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 1024;
    let uri = benchmark_uri(64 * 1024, "benchmark-label");

    let mut warm = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Data);
    legacy_refresh_loaded_meta_identity(&mut warm, &uri, AssetKind::Data);
    refresh_loaded_meta_identity(&mut warm, &uri, AssetKind::Data);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_meta = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Data);
        let optimized_meta = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Data);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_refresh(
                legacy_meta,
                &uri,
                ITERATIONS,
                legacy_refresh_loaded_meta_identity,
            ));
            optimized_samples.push(measure_refresh(
                optimized_meta,
                &uri,
                ITERATIONS,
                refresh_loaded_meta_identity,
            ));
        } else {
            optimized_samples.push(measure_refresh(
                optimized_meta,
                &uri,
                ITERATIONS,
                refresh_loaded_meta_identity,
            ));
            legacy_samples.push(measure_refresh(
                legacy_meta,
                &uri,
                ITERATIONS,
                legacy_refresh_loaded_meta_identity,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME286_MATCHING_META_IDENTITY_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_uri(path_bytes: usize, label: &str) -> AssetUri {
    AssetUri::parse(&format!("res://{}#{label}", "a".repeat(path_bytes)))
        .expect("benchmark asset URI")
}

fn legacy_refresh_loaded_meta_identity(
    meta: &mut AssetMetaDocument,
    uri: &AssetUri,
    kind: AssetKind,
) {
    meta.url = uri.clone();
    meta.asset_kind = kind;
}

fn measure_refresh(
    mut meta: AssetMetaDocument,
    uri: &AssetUri,
    iterations: usize,
    refresh: fn(&mut AssetMetaDocument, &AssetUri, AssetKind),
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        refresh(black_box(&mut meta), black_box(uri), AssetKind::Data);
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(meta);
    elapsed
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
