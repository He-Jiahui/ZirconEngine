use std::hint::black_box;
use std::time::Instant;

use super::validate_runtime_plugin_package_root_field;

const CHECKS_PER_SAMPLE: usize = 16_384;
const ROOT_BYTES: usize = 4096;
const SAMPLE_PAIRS: usize = 31;

fn legacy_validate(field_name: &str, root: &str, diagnostics: &mut Vec<String>) -> bool {
    if root.trim().is_empty() || root.trim() != root {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be non-empty and trimmed"
        ));
        return false;
    }
    true
}

fn measure(root: &str, optimized: bool) -> u128 {
    let mut diagnostics = Vec::new();
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        valid += usize::from(if optimized {
            validate_runtime_plugin_package_root_field("content", black_box(root), &mut diagnostics)
        } else {
            legacy_validate("content", black_box(root), &mut diagnostics)
        });
    }
    black_box((valid, diagnostics.len()));
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

#[test]
fn optimization_batch_20260829bf_runtime333_single_trim_root_preserves_results() {
    for root in [
        "content/assets",
        "",
        " ",
        " content/assets",
        "content/assets ",
        "\u{2003}content/assets",
    ] {
        let mut legacy = Vec::new();
        let mut optimized = Vec::new();
        let legacy_valid = legacy_validate("content", root, &mut legacy);
        let optimized_valid =
            validate_runtime_plugin_package_root_field("content", root, &mut optimized);
        assert_eq!(
            (optimized_valid, optimized),
            (legacy_valid, legacy),
            "{root:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bf_runtime333_root_validation_uses_length_check() {
    let source = include_str!("../field.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("root.trim()").count(), 1);
    assert!(production.contains("trimmed.len() != root.len()"));
    assert!(!production.contains("root.trim() != root"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bf_runtime333_single_trim_package_root_bench() {
    let root = format!("content/{}", "x".repeat(ROOT_BYTES - "content/".len()));
    assert_eq!(root.len(), ROOT_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&root, false));
            optimized_samples.push(measure(&root, true));
        } else {
            optimized_samples.push(measure(&root, true));
            legacy_samples.push(measure(&root, false));
        }
    }

    let baseline_p50_ns = percentile(&legacy_samples, 50);
    let candidate_p50_ns = percentile(&optimized_samples, 50);
    let baseline_p95_ns = percentile(&legacy_samples, 95);
    let candidate_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME333_SINGLE_TRIM_PACKAGE_ROOT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} root_bytes={ROOT_BYTES} \
legacy_full_root_comparisons=1 optimized_full_root_comparisons=0 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
