use std::hint::black_box;
use std::time::Instant;

use super::{is_lowercase_runtime_plugin_token, validate_runtime_plugin_package_token_charset};

const CHECKS_PER_SAMPLE: usize = 16_384;
const SAMPLE_PAIRS: usize = 31;
const TOKEN_BYTES: usize = 4096;

fn legacy_validate(field_name: &str, value: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() || value.trim() != value || !is_lowercase_runtime_plugin_token(value)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
}

fn measure(value: &str, optimized: bool) -> u128 {
    let mut diagnostics = Vec::new();
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            validate_runtime_plugin_package_token_charset(
                "package_id",
                black_box(value),
                &mut diagnostics,
            );
        } else {
            legacy_validate("package_id", black_box(value), &mut diagnostics);
        }
    }
    black_box(diagnostics.len());
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
fn optimization_batch_20260829bg_runtime334_single_scan_tokens_preserve_diagnostics() {
    for value in [
        "runtime_plugin_2",
        "",
        " ",
        " runtime_plugin",
        "runtime_plugin ",
        "RuntimePlugin",
        "runtime-plugin",
        "runtime_\u{4f8b}",
    ] {
        let mut baseline = Vec::new();
        let mut candidate = Vec::new();
        legacy_validate("package_id", value, &mut baseline);
        validate_runtime_plugin_package_token_charset("package_id", value, &mut candidate);
        assert_eq!(candidate, baseline, "{value:?}");
    }
}

#[test]
fn optimization_batch_20260829bg_runtime334_token_charset_uses_predicate_once() {
    let source = include_str!("../charset.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(
        production
            .matches("is_lowercase_runtime_plugin_token(value)")
            .count(),
        1
    );
    assert!(!production.contains(".trim()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bg_runtime334_single_scan_package_token_bench() {
    let value = "a".repeat(TOKEN_BYTES);
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&value, false));
            candidate_samples.push(measure(&value, true));
        } else {
            candidate_samples.push(measure(&value, true));
            baseline_samples.push(measure(&value, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "RUNTIME334_SINGLE_SCAN_PACKAGE_TOKEN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} token_bytes={TOKEN_BYTES} \
baseline_full_token_scans=2 candidate_full_token_scans=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
