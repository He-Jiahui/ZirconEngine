use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{normalize_include_token, path_matches_normalized_include_token};

const PERFORMANCE_MARKER: &str = "RUNTIME140_SHADING_INCLUDE_TOKEN_HOIST_BENCH_V1";

#[test]
fn optimization_batch_20260826cw_runtime140_normalized_token_matches_supported_locator_shapes() {
    let token = normalize_include_token(" ZR_Shading_Toon.WGSL ");

    assert!(path_matches_normalized_include_token(
        "package://toon/Shaders/ZR_SHADING_TOON.wgsl",
        &token
    ));
    assert!(path_matches_normalized_include_token(
        r"package:\toon\Shaders\zr_shading_toon.WGSL",
        &token
    ));
    assert!(path_matches_normalized_include_token(
        "zr_shading_toon.wgsl",
        &token
    ));
    assert!(!path_matches_normalized_include_token(
        "package://toon/shaders/not_zr_shading_toon.wgsl",
        &token
    ));
}

#[test]
fn optimization_batch_20260826cw_runtime140_include_scan_hoists_token_and_avoids_suffix_format() {
    let source = include_str!("../include_sources.rs")
        .split_once("#[cfg(test)]")
        .expect("include source test boundary should exist")
        .0;
    let resolver = source
        .split_once("fn resolve_include_source(")
        .expect("resolver should exist")
        .1
        .split_once("fn record_matches_include_token(")
        .expect("record matcher should follow resolver")
        .0;

    assert_eq!(
        resolver.matches("normalize_include_token(token)").count(),
        1
    );
    assert!(source.contains("strip_suffix(normalized_token)"));
    assert!(!source.contains("format!(\"/{token}\")"));
}

#[test]
#[ignore = "release-only shading include token hoist performance gate"]
fn optimization_batch_20260826cw_runtime140_include_token_hoist_performance_evidence() {
    const PATH_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME140_SHADING_INCLUDE_TOKEN_HOIST_BENCH_V1"
    );
    let token = " ZR_Shading_Toon.WGSL ";
    let paths = (0..PATH_COUNT)
        .map(|index| {
            format!(
                "package:\\plugin_{index:08}\\Shaders\\Generated\\ZR_Shading_Model_{index:08}.WGSL"
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(legacy_scan(&paths, token));
        black_box(optimized_scan(&paths, token));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| legacy_scan(&paths, token)));
            optimized_samples.push(measure(|| optimized_scan(&paths, token)));
        } else {
            optimized_samples.push(measure(|| optimized_scan(&paths, token)));
            legacy_samples.push(measure(|| legacy_scan(&paths, token)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} paths={PATH_COUNT} samples={SAMPLE_COUNT} legacy_token_normalizations={PATH_COUNT} optimized_token_normalizations=1 legacy_suffix_allocations={PATH_COUNT} optimized_suffix_allocations=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "hoisted include token P95 {optimized_p95_ns}ns must be at most 70% of per-record normalization P95 {legacy_p95_ns}ns"
    );
}

fn legacy_scan(paths: &[String], token: &str) -> usize {
    paths
        .iter()
        .filter(|path| {
            let token = normalize_include_token(token);
            let path = normalize_include_token(path);
            path == token || path.ends_with(&format!("/{token}"))
        })
        .count()
}

fn optimized_scan(paths: &[String], token: &str) -> usize {
    let token = normalize_include_token(token);
    paths
        .iter()
        .filter(|path| path_matches_normalized_include_token(path, &token))
        .count()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
