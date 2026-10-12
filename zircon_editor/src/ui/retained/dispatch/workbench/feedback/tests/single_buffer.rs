use std::hint::black_box;
use std::time::{Duration, Instant};

use super::input_summary;

const PERFORMANCE_MARKER: &str = "EDITOR891_LIVE_INPUT_SUMMARY_SINGLE_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 101;
const SUMMARIES_PER_SAMPLE: usize = 4_096;

#[test]
fn editor891_live_input_summary_single_buffer_preserves_exact_text() {
    for values in [
        &[][..],
        &["   "][..],
        &[" alpha "][..],
        &["alpha", "beta"][..],
        &["alpha", " ", "beta", "gamma", "ignored"][..],
        &["\u{6e32}\u{67d3}", "editor", "runtime", "ignored"][..],
    ] {
        assert_eq!(
            optimized_input_summary(values),
            legacy_input_summary(values),
            "values={values:?}"
        );
    }
    assert_eq!(
        optimized_input_summary(&["alpha", " ", "beta", "gamma", "ignored"]),
        Some("Inputs: alpha | beta | gamma".to_string())
    );
}

#[test]
fn editor891_live_input_summary_single_buffer_uses_exact_capacity_for_uneven_values() {
    let summary = optimized_input_summary(&["a", "a substantially longer value", "中"])
        .expect("non-empty inputs should produce a summary");

    assert_eq!(summary.capacity(), summary.len());
}

#[test]
#[ignore = "release-only live input summary performance gate"]
fn editor891_live_input_summary_single_buffer_release_performance() {
    let values = [
        "  shader_target_desktop_vulkan  ",
        "material_quality_ultra",
        "pipeline_cache_enabled",
        "ignored_after_three",
    ];
    assert_eq!(
        optimized_input_summary(&values),
        legacy_input_summary(&values)
    );

    for _ in 0..8 {
        black_box(render_batch(&values, legacy_input_summary));
        black_box(render_batch(&values, optimized_input_summary));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| render_batch(&values, legacy_input_summary)));
            optimized_samples.push(measure(|| render_batch(&values, optimized_input_summary)));
        } else {
            optimized_samples.push(measure(|| render_batch(&values, optimized_input_summary)));
            legacy_samples.push(measure(|| render_batch(&values, legacy_input_summary)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} sample_pairs={SAMPLE_PAIRS} summaries_per_sample={SUMMARIES_PER_SAMPLE} visible_values_per_summary=3 legacy_reference_slots_per_sample=12288 optimized_reference_slots_per_sample=0 legacy_join_outputs_per_sample=4096 optimized_join_outputs_per_sample=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(110),
        "single-buffer P95 {optimized_p95_ns}ns must be at most 110% of collect/join P95 {legacy_p95_ns}ns"
    );
}

fn optimized_input_summary(values: &[&str]) -> Option<String> {
    input_summary(values.iter().copied())
}

fn legacy_input_summary(values: &[&str]) -> Option<String> {
    let values = values
        .iter()
        .copied()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(3)
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| format!("Inputs: {}", values.join(" | ")))
}

fn render_batch(values: &[&str], render: fn(&[&str]) -> Option<String>) -> usize {
    (0..SUMMARIES_PER_SAMPLE)
        .map(|_| {
            black_box(render(black_box(values)))
                .as_deref()
                .map_or(0, str::len)
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
