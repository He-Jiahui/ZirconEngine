use std::hint::black_box;
use std::time::{Duration, Instant};

use super::camel_to_snake_segment;

const PERFORMANCE_MARKER: &str = "EDITOR87_BINDING_SNAKE_CASE_IN_PLACE_TRIM_BENCH_V1";

#[test]
fn optimization_batch_20260826cx_editor87_in_place_trim_preserves_legacy_snake_case() {
    for binding in [
        "SetValue",
        "/SetValue/",
        "--Set--Value--",
        "HTTPServer2Value",
        "already_snake_case",
        "\u{00c5}ngstromValue",
        "___",
        "",
    ] {
        assert_eq!(
            camel_to_snake_segment(binding),
            legacy_camel_to_snake_segment(binding),
            "{binding}"
        );
    }
}

#[test]
fn optimization_batch_20260826cx_editor87_snake_case_reuses_output_buffer_for_trim() {
    let source = include_str!("../bindings.rs")
        .split_once("#[cfg(test)]")
        .expect("binding test boundary should exist")
        .0;
    let normalization = source
        .split_once("fn camel_to_snake_segment")
        .expect("snake-case helper should exist")
        .1;

    assert!(normalization.contains("String::with_capacity(value.len())"));
    assert!(normalization.contains("output.pop()"));
    assert!(!normalization.contains("trim_matches('_').to_string()"));
}

#[test]
#[ignore = "release-only binding snake-case performance gate"]
fn optimization_batch_20260826cx_editor87_in_place_trim_performance_evidence() {
    const BINDING_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR87_BINDING_SNAKE_CASE_IN_PLACE_TRIM_BENCH_V1"
    );
    let bindings = (0..BINDING_COUNT)
        .map(|index| {
            format!("--MaterialPreviewRuntimeBindingAction{index:08}SelectedValueChanged--")
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(normalize_batch(&bindings, legacy_camel_to_snake_segment));
        black_box(normalize_batch(&bindings, camel_to_snake_segment));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                normalize_batch(&bindings, legacy_camel_to_snake_segment)
            }));
            optimized_samples.push(measure(|| {
                normalize_batch(&bindings, camel_to_snake_segment)
            }));
        } else {
            optimized_samples.push(measure(|| {
                normalize_batch(&bindings, camel_to_snake_segment)
            }));
            legacy_samples.push(measure(|| {
                normalize_batch(&bindings, legacy_camel_to_snake_segment)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} bindings={BINDING_COUNT} samples={SAMPLE_COUNT} legacy_allocations_per_binding=2 optimized_allocations_per_binding=1"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place trim P95 {optimized_p95_ns}ns must be at most 70% of copied-trim P95 {legacy_p95_ns}ns"
    );
}

fn legacy_camel_to_snake_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

fn normalize_batch(bindings: &[String], normalize: fn(&str) -> String) -> usize {
    bindings
        .iter()
        .map(|binding| black_box(normalize(black_box(binding))).len())
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
