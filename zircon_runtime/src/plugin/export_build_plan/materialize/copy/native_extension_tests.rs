use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::is_native_dynamic_artifact;

const PERFORMANCE_MARKER: &str = "RUNTIME141_NATIVE_EXTENSION_BORROWED_DISPATCH_BENCH_V1";

#[test]
fn optimization_batch_20260826cx_runtime141_native_extension_dispatch_preserves_supported_set() {
    for extension in [
        "dll", "DLL", "so", "SO", "dylib", "DyLiB", "pdb", "PDB", "dbg", "DBG", "dsym", "DSYM",
    ] {
        let path = PathBuf::from(format!("native/runtime_artifact.{extension}"));
        assert!(is_native_dynamic_artifact(&path), "{extension}");
    }

    for path in [
        "native/runtime_artifact",
        "native/runtime_artifact.exe",
        "native/runtime_artifact.dll.backup",
        "native/.dll",
    ] {
        assert!(!is_native_dynamic_artifact(Path::new(path)), "{path}");
    }
}

#[test]
fn optimization_batch_20260826cx_runtime141_native_extension_dispatch_avoids_owned_lowercase() {
    let source = include_str!("../copy.rs")
        .split_once("#[cfg(test)]")
        .expect("copy test boundary should exist")
        .0;
    let dispatch = source
        .split_once("fn is_native_dynamic_artifact")
        .expect("native extension dispatch should exist")
        .1;

    assert!(dispatch.contains("match extension.len()"));
    assert!(dispatch.contains("extension.eq_ignore_ascii_case"));
    assert!(!dispatch.contains("extension.to_ascii_lowercase()"));
}

#[test]
#[ignore = "release-only native extension dispatch performance gate"]
fn optimization_batch_20260826cx_runtime141_native_extension_dispatch_performance_evidence() {
    const PATH_COUNT: usize = 8_192;
    const ITERATIONS_PER_SAMPLE: usize = 16;
    const SAMPLE_COUNT: usize = 17;
    const PROBE_COUNT: usize = PATH_COUNT * ITERATIONS_PER_SAMPLE;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME141_NATIVE_EXTENSION_BORROWED_DISPATCH_BENCH_V1"
    );
    let paths = (0..PATH_COUNT)
        .map(|index| PathBuf::from(format!("native/bin/runtime_module_{index:08}.DLL")))
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(legacy_dispatch_batch(&paths, ITERATIONS_PER_SAMPLE));
        black_box(optimized_dispatch_batch(&paths, ITERATIONS_PER_SAMPLE));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                legacy_dispatch_batch(&paths, ITERATIONS_PER_SAMPLE)
            }));
            optimized_samples.push(measure(|| {
                optimized_dispatch_batch(&paths, ITERATIONS_PER_SAMPLE)
            }));
        } else {
            optimized_samples.push(measure(|| {
                optimized_dispatch_batch(&paths, ITERATIONS_PER_SAMPLE)
            }));
            legacy_samples.push(measure(|| {
                legacy_dispatch_batch(&paths, ITERATIONS_PER_SAMPLE)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} paths={PATH_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} probes={PROBE_COUNT} samples={SAMPLE_COUNT} legacy_extension_allocations={PROBE_COUNT} optimized_extension_allocations=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed extension dispatch P95 {optimized_p95_ns}ns must be at most 70% of lowercase-allocation P95 {legacy_p95_ns}ns"
    );
}

fn legacy_is_native_dynamic_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "dll" | "so" | "dylib" | "pdb" | "dbg" | "dsym"
    )
}

fn legacy_dispatch_batch(paths: &[PathBuf], iterations: usize) -> usize {
    (0..iterations)
        .map(|_| {
            paths
                .iter()
                .filter(|path| legacy_is_native_dynamic_artifact(black_box(path)))
                .count()
        })
        .sum()
}

fn optimized_dispatch_batch(paths: &[PathBuf], iterations: usize) -> usize {
    (0..iterations)
        .map(|_| {
            paths
                .iter()
                .filter(|path| is_native_dynamic_artifact(black_box(path)))
                .count()
        })
        .sum()
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
