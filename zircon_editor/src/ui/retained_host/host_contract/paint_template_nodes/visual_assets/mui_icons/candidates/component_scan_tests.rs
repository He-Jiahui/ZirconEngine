use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::is_module_path;

const SAMPLE_PAIRS: usize = 21;
const OPERATIONS_PER_SAMPLE: usize = 1_024;

#[test]
fn optimization_batch_20260826ha_editor193_preserves_mui_module_path_matching() {
    assert!(is_module_path(
        &PathBuf::from("workspace")
            .join("mui-icons-material")
            .join("lib")
            .join("Add.JS")
    ));
    assert!(is_module_path(
        &PathBuf::from("mui-icons-material")
            .join("mui-icons-material")
            .join("lib")
            .join("Close.js")
    ));
    assert!(!is_module_path(
        &PathBuf::from("workspace")
            .join("mui-icons-material-extra")
            .join("lib")
            .join("Add.js")
    ));
    assert!(!is_module_path(
        &PathBuf::from("workspace")
            .join("mui-icons-material")
            .join("src")
            .join("Add.js")
    ));
    assert!(!is_module_path(
        &PathBuf::from("workspace")
            .join("mui-icons-material")
            .join("lib")
            .join("Add.ts")
    ));
}

#[test]
fn optimization_batch_20260826ha_editor193_avoids_full_path_string_materialization() {
    let source = include_str!("../candidates.rs");
    let start = source
        .find("fn is_module_path(")
        .expect("is_module_path function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("path.components()"));
    assert!(!body.contains("to_string_lossy"));
    assert!(!body.contains("replace("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826ha_editor193_mui_icon_component_scan_release_benchmark() {
    let mut path = PathBuf::from("workspace");
    for index in 0..96 {
        path.push(format!("package-{index:03}"));
    }
    path.push("mui-icons-material");
    path.push("lib");
    path.push("Add.js");
    assert!(legacy_is_module_path(&path));
    assert!(is_module_path(&path));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_is_module_path(black_box(&path)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(is_module_path(black_box(&path)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR193_MUI_ICON_COMPONENT_SCAN_BENCH_V1 components=99 \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_is_module_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
        && path
            .to_string_lossy()
            .replace('\\', "/")
            .contains("/mui-icons-material/lib/")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
