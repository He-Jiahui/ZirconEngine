use std::hint::black_box;
use std::time::Instant;

use super::is_obj_mesh_extension;

const PERF_MARKER: &str = "RUNTIME135_OBJ_EXTENSION_BORROWED_DISPATCH_BENCH_V1";

#[test]
fn optimization_batch_20260826cr_runtime_obj_extension_dispatch_preserves_case_aliases() {
    for extension in ["obj", "OBJ", "Obj", "oBj"] {
        assert!(is_obj_mesh_extension(extension), "extension={extension}");
    }
    for extension in ["fbx", "objx", "obj\u{00e9}", ""] {
        assert!(!is_obj_mesh_extension(extension), "extension={extension}");
    }
}

#[test]
fn optimization_batch_20260826cr_runtime_obj_extension_dispatch_source_contract() {
    let source = include_str!("../mesh.rs");

    assert!(source.contains("if is_obj_mesh_extension(extension)"));
    assert!(source.contains("extension.eq_ignore_ascii_case(\"obj\")"));
    assert!(source.contains("extension: extension.to_ascii_lowercase()"));
    assert!(!source.contains("match extension.as_str()"));
    assert_eq!(
        PERF_MARKER,
        "RUNTIME135_OBJ_EXTENSION_BORROWED_DISPATCH_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826cr_runtime_obj_extension_dispatch_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 240_000;
    let extensions = ["obj", "OBJ", "Obj", "oBj"];

    black_box(measure_legacy(&extensions, CHECKS_PER_SAMPLE / 10));
    black_box(measure_optimized(&extensions, CHECKS_PER_SAMPLE / 10));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&extensions, CHECKS_PER_SAMPLE));
            optimized_ns.push(measure_optimized(&extensions, CHECKS_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_optimized(&extensions, CHECKS_PER_SAMPLE));
            legacy_ns.push(measure_legacy(&extensions, CHECKS_PER_SAMPLE));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} aliases=4 order=alternating_legacy_first_even legacy_normalized_string_allocations_per_sample={CHECKS_PER_SAMPLE} optimized_normalized_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "borrowed OBJ extension dispatch must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(extensions: &[&str], checks: usize) -> u128 {
    measure(extensions, checks, |extension| {
        extension.to_ascii_lowercase() == "obj"
    })
}

fn measure_optimized(extensions: &[&str], checks: usize) -> u128 {
    measure(extensions, checks, is_obj_mesh_extension)
}

fn measure(extensions: &[&str], checks: usize, predicate: fn(&str) -> bool) -> u128 {
    let mut matches = 0usize;
    let started = Instant::now();
    for index in 0..checks {
        matches += usize::from(predicate(black_box(extensions[index % extensions.len()])));
    }
    black_box(matches);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
