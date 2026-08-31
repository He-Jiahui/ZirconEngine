use std::hint::black_box;
use std::time::Instant;

use super::{property_axis_values, PropertyAxisValue};

const PERF_MARKER: &str = "EDITOR83_PROPERTY_AXIS_BORROWED_TOKEN_BUFFER_BENCH_V1";

#[test]
fn optimization_batch_20260826ct_editor_property_axis_buffer_preserves_grouping() {
    let values = property_axis_values("ignored X 10 px auto Y Z -2.5 rem W 1 2 3");

    assert_eq!(
        values,
        vec![
            PropertyAxisValue {
                axis: "X".to_owned(),
                value: "10 px auto".to_owned(),
            },
            PropertyAxisValue {
                axis: "Z".to_owned(),
                value: "-2.5 rem".to_owned(),
            },
            PropertyAxisValue {
                axis: "W".to_owned(),
                value: "1 2 3".to_owned(),
            },
        ]
    );
}

#[test]
fn optimization_batch_20260826ct_editor_property_axis_buffer_source_contract() {
    let source = include_str!("../template_property_axis_values.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("property axis production source");

    assert!(production.contains("current_value.push(token);"));
    assert!(production.contains("current_value: &mut Vec<&str>"));
    assert!(!production.contains("current_value.push(token.to_string())"));
    assert!(!production.contains("current_value: &mut Vec<String>"));
    assert_eq!(
        PERF_MARKER,
        "EDITOR83_PROPERTY_AXIS_BORROWED_TOKEN_BUFFER_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826ct_editor_property_axis_buffer_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const AXIS_GROUPS: usize = 256;
    const PARSES_PER_SAMPLE: usize = 160;
    let input = (0..AXIS_GROUPS)
        .map(|index| format!("{} {index} px auto", ["X", "Y", "Z", "W"][index % 4]))
        .collect::<Vec<_>>()
        .join("   ");

    black_box(measure_legacy(&input, PARSES_PER_SAMPLE / 10));
    black_box(measure_optimized(&input, PARSES_PER_SAMPLE / 10));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&input, PARSES_PER_SAMPLE));
            optimized_ns.push(measure_optimized(&input, PARSES_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_optimized(&input, PARSES_PER_SAMPLE));
            legacy_ns.push(measure_legacy(&input, PARSES_PER_SAMPLE));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} axis_groups={AXIS_GROUPS} parses_per_sample={PARSES_PER_SAMPLE} value_tokens_per_group=3 order=alternating_legacy_first_even legacy_owned_value_tokens_per_sample={} optimized_owned_value_tokens_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}",
        AXIS_GROUPS * 3 * PARSES_PER_SAMPLE
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "borrowed property-axis token buffering must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_property_axis_values(value: &str) -> Vec<PropertyAxisValue> {
    let mut values = Vec::new();
    let mut current_axis: Option<String> = None;
    let mut current_value = Vec::new();
    for token in value.split_whitespace() {
        if matches!(token, "X" | "Y" | "Z" | "W") {
            legacy_push(&mut values, &mut current_axis, &mut current_value);
            current_axis = Some(token.to_owned());
        } else if current_axis.is_some() {
            current_value.push(token.to_owned());
        }
    }
    legacy_push(&mut values, &mut current_axis, &mut current_value);
    values
}

fn legacy_push(
    values: &mut Vec<PropertyAxisValue>,
    current_axis: &mut Option<String>,
    current_value: &mut Vec<String>,
) {
    let Some(axis) = current_axis.take() else {
        return;
    };
    let value = current_value.join(" ");
    current_value.clear();
    if !value.is_empty() {
        values.push(PropertyAxisValue { axis, value });
    }
}

fn measure_legacy(input: &str, parses: usize) -> u128 {
    measure(input, parses, legacy_property_axis_values)
}

fn measure_optimized(input: &str, parses: usize) -> u128 {
    measure(input, parses, property_axis_values)
}

fn measure(input: &str, parses: usize, parse: fn(&str) -> Vec<PropertyAxisValue>) -> u128 {
    let mut checksum = 0usize;
    let started = Instant::now();
    for _ in 0..parses {
        let values = parse(black_box(input));
        checksum = checksum.wrapping_add(
            values
                .iter()
                .map(|value| value.axis.len() + value.value.len())
                .sum::<usize>(),
        );
        black_box(values);
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
