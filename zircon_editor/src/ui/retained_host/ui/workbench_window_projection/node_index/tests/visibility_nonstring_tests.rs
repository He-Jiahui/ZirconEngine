use std::hint::black_box;
use std::time::Instant;

use crate::ui::template_runtime::RetainedUiHostValue;

use super::{normalized_ascii_eq, visibility_is};

const SAMPLE_PAIRS: usize = 17;
const VALUE_COUNT: usize = 65_536;

fn legacy_visibility_is(value: Option<&RetainedUiHostValue>, expected: &str) -> bool {
    let Some(value) = value else {
        return expected == "visible";
    };
    match value {
        RetainedUiHostValue::String(value) => normalized_ascii_eq(value, expected),
        RetainedUiHostValue::Integer(value) => normalized_ascii_eq(&value.to_string(), expected),
        RetainedUiHostValue::Float(value) => normalized_ascii_eq(&value.to_string(), expected),
        RetainedUiHostValue::Bool(value) => normalized_ascii_eq(&value.to_string(), expected),
        RetainedUiHostValue::Datetime(_)
        | RetainedUiHostValue::Array(_)
        | RetainedUiHostValue::Table(_) => expected == "visible",
    }
}

#[test]
fn optimization_batch_20260915_editor789_visibility_is_preserves_non_string_results() {
    for value in [
        RetainedUiHostValue::Integer(1),
        RetainedUiHostValue::Float(1.5),
        RetainedUiHostValue::Bool(true),
        RetainedUiHostValue::Bool(false),
        RetainedUiHostValue::Datetime("2026-09-15T00:00:00Z".to_string()),
        RetainedUiHostValue::Array(Vec::new()),
        RetainedUiHostValue::Table(Default::default()),
    ] {
        for expected in ["visible", "collapsed", "hittestinvisible"] {
            assert_eq!(
                visibility_is(Some(&value), expected),
                legacy_visibility_is(Some(&value), expected),
                "value={value:?} expected={expected}"
            );
        }
    }
    assert!(visibility_is(
        Some(&RetainedUiHostValue::String(
            "Self_HitTestInvisible".to_string()
        )),
        "selfhittestinvisible"
    ));
}

#[test]
fn optimization_batch_20260915_editor789_visibility_is_avoids_scalar_formatting() {
    let source = include_str!("../../node_index.rs");
    let start = source
        .find("fn visibility_is(")
        .expect("visibility function");
    let end = source[start..]
        .find("\n}\n\nfn normalized_ascii_eq")
        .map(|offset| start + offset)
        .expect("visibility function end");
    let production = &source[start..end];
    assert!(production.contains("RetainedUiHostValue::Integer(_)"));
    assert!(production.contains("RetainedUiHostValue::Float(_)"));
    assert!(production.contains("RetainedUiHostValue::Bool(_) => false"));
    assert!(!production.contains("to_string()"));
}

fn measure_nanos(run: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos().max(1)
}

fn legacy_scalar_scan(values: &[RetainedUiHostValue]) -> usize {
    values
        .iter()
        .filter(|value| legacy_visibility_is(Some(value), "visible"))
        .count()
}

fn optimized_scalar_scan(values: &[RetainedUiHostValue]) -> usize {
    values
        .iter()
        .filter(|value| visibility_is(Some(value), "visible"))
        .count()
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260915_editor789_visibility_is_p95() {
    let values = (0..VALUE_COUNT)
        .map(|index| RetainedUiHostValue::Integer(index as i64))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_nanos(|| legacy_scalar_scan(black_box(&values))));
            optimized_samples.push(measure_nanos(|| optimized_scalar_scan(black_box(&values))));
        } else {
            optimized_samples.push(measure_nanos(|| optimized_scalar_scan(black_box(&values))));
            legacy_samples.push(measure_nanos(|| legacy_scalar_scan(black_box(&values))));
        }
    }
    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR789_WORKBENCH_NODE_VISIBILITY_NON_STRING_BENCH_V1 values={VALUE_COUNT} \
         legacy_scalar_string_formats={VALUE_COUNT} optimized_scalar_string_formats=0 \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert_eq!(legacy_scalar_scan(&values), 0);
    assert_eq!(optimized_scalar_scan(&values), 0);
    assert!(legacy_p95 > 0);
    assert!(optimized_p95 > 0);
}
