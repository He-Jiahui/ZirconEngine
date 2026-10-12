use std::hint::black_box;
use std::time::Instant;

use super::{
    format_primary_window_diagnostic, format_scale_factor_override_diagnostic, PrimaryWindowHandle,
};

const SAMPLE_PAIRS: usize = 21;
const FORMATS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_20260826dr_runtime161_window_optional_diagnostics_preserve_values() {
    assert_eq!(
        format_primary_window_diagnostic(Some(PrimaryWindowHandle::new(73))),
        "window.primary_window=73"
    );
    assert_eq!(
        format_primary_window_diagnostic(None),
        "window.primary_window=none"
    );
    assert_eq!(
        format_scale_factor_override_diagnostic(Some(1.25)),
        "window.scale_factor_override=1.25"
    );
    assert_eq!(
        format_scale_factor_override_diagnostic(None),
        "window.scale_factor_override=none"
    );
}

#[test]
fn optimization_batch_20260826dr_runtime161_window_optional_diagnostics_format_directly() {
    let source = include_str!("../descriptor.rs");
    assert!(source.contains("format_primary_window_diagnostic(self.primary_window)"));
    assert!(source.contains(
        "format_scale_factor_override_diagnostic(self.resolution.scale_factor_override())"
    ));
    assert!(source.contains(
        "Some(scale_factor) => format!(\"window.scale_factor_override={scale_factor}\")"
    ));
    assert!(!source.contains(".map(|scale_factor| scale_factor.to_string())"));
    assert!(!source.contains(".map(|handle| handle.raw().to_string())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dr_runtime161_window_optional_diagnostics_direct_format_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_scale_factor_override));
            optimized_samples.push(measure(format_scale_factor_override_diagnostic));
        } else {
            optimized_samples.push(measure(format_scale_factor_override_diagnostic));
            legacy_samples.push(measure(legacy_scale_factor_override));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME161_WINDOW_OPTIONAL_DIAGNOSTICS_DIRECT_FORMAT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
formats_per_sample={FORMATS_PER_SAMPLE} legacy_allocations_per_format=2 \
optimized_allocations_per_format=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct optional window diagnostic P95 {optimized_p95_ns}ns must be at most 70% of intermediate-string formatting P95 {legacy_p95_ns}ns"
    );
}

fn legacy_scale_factor_override(scale_factor: Option<f32>) -> String {
    format!(
        "window.scale_factor_override={}",
        scale_factor
            .map(|scale_factor| scale_factor.to_string())
            .unwrap_or_else(|| "none".to_string())
    )
}

fn measure(render: fn(Option<f32>) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORMATS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(None))).len();
    }
    black_box(checksum);
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
