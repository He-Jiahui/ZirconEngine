use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::template::UiRootClassPolicy;

use super::parse_root_class_policy;

const PERFORMANCE_MARKER: &str = "EDITOR90_ROOT_CLASS_POLICY_BORROWED_PARSE_BENCH_V1";

#[test]
fn optimization_batch_20260826da_editor90_root_class_policy_preserves_supported_aliases() {
    for value in ["append_only", " APPEND_ONLY ", "Append-Only", "appendonly"] {
        assert_eq!(
            parse_root_class_policy(value),
            Some(UiRootClassPolicy::AppendOnly),
            "{value}"
        );
    }
    assert_eq!(
        parse_root_class_policy(" CLOSED "),
        Some(UiRootClassPolicy::Closed)
    );
    assert_eq!(parse_root_class_policy("open"), None);
}

#[test]
fn optimization_batch_20260826da_editor90_root_class_policy_avoids_normalized_string() {
    let source = include_str!("../root_class_policy_state.rs")
        .split_once("#[cfg(test)]")
        .expect("root policy test boundary should exist")
        .0;
    let parser = source
        .split_once("fn parse_root_class_policy")
        .expect("root policy parser should exist")
        .1;

    assert!(parser.contains("eq_ignore_ascii_case"));
    assert!(!parser.contains("to_ascii_lowercase()"));
    assert!(!parser.contains("replace('-', \"_\")"));
}

#[test]
#[ignore = "release-only root class policy parse performance gate"]
fn optimization_batch_20260826da_editor90_root_class_policy_performance_evidence() {
    const VALUE_COUNT: usize = 16_384;
    const ITERATIONS_PER_SAMPLE: usize = 16;
    const SAMPLE_COUNT: usize = 17;
    const PARSE_COUNT: usize = VALUE_COUNT * ITERATIONS_PER_SAMPLE;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR90_ROOT_CLASS_POLICY_BORROWED_PARSE_BENCH_V1"
    );
    let values = (0..VALUE_COUNT)
        .map(|index| {
            if index % 2 == 0 {
                " Append_Only ".to_string()
            } else {
                " append_only ".to_string()
            }
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(parse_batch(
            &values,
            ITERATIONS_PER_SAMPLE,
            legacy_parse_root_class_policy,
        ));
        black_box(parse_batch(
            &values,
            ITERATIONS_PER_SAMPLE,
            parse_root_class_policy,
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                parse_batch(
                    &values,
                    ITERATIONS_PER_SAMPLE,
                    legacy_parse_root_class_policy,
                )
            }));
            optimized_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_root_class_policy)
            }));
        } else {
            optimized_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_root_class_policy)
            }));
            legacy_samples.push(measure(|| {
                parse_batch(
                    &values,
                    ITERATIONS_PER_SAMPLE,
                    legacy_parse_root_class_policy,
                )
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} values={VALUE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} parses={PARSE_COUNT} samples={SAMPLE_COUNT} legacy_allocations_per_parse=2 optimized_allocations_per_parse=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed root policy parse P95 {optimized_p95_ns}ns must be at most 70% of normalized-string P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_root_class_policy(value: &str) -> Option<UiRootClassPolicy> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "append_only" | "appendonly" => Some(UiRootClassPolicy::AppendOnly),
        "closed" => Some(UiRootClassPolicy::Closed),
        _ => None,
    }
}

fn parse_batch(
    values: &[String],
    iterations: usize,
    parse: fn(&str) -> Option<UiRootClassPolicy>,
) -> usize {
    (0..iterations)
        .map(|_| {
            values
                .iter()
                .filter(|value| parse(black_box(value)).is_some())
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
