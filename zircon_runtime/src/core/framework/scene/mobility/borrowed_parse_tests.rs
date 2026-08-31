use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{reflection::parse_mobility, Mobility};

const PERFORMANCE_MARKER: &str = "RUNTIME144_MOBILITY_BORROWED_ENUM_PARSE_BENCH_V1";

#[test]
fn optimization_batch_20260826da_runtime144_mobility_parser_preserves_enum_semantics() {
    assert_eq!(parse_mobility(" dynamic "), Some(Mobility::Dynamic));
    assert_eq!(parse_mobility("DyNaMiC"), Some(Mobility::Dynamic));
    assert_eq!(parse_mobility(" STATIC "), Some(Mobility::Static));
    assert_eq!(parse_mobility("stationary"), None);
    assert_eq!(parse_mobility("  "), None);
}

#[test]
fn optimization_batch_20260826da_runtime144_mobility_parser_avoids_owned_lowercase() {
    let source = include_str!("../mobility.rs")
        .split_once("#[cfg(test)]")
        .expect("mobility test boundary should exist")
        .0;
    let parser = source
        .split_once("fn parse_mobility")
        .expect("mobility parser should exist")
        .1;

    assert!(parser.contains("eq_ignore_ascii_case"));
    assert!(!parser.contains("to_ascii_lowercase()"));
}

#[test]
#[ignore = "release-only mobility enum parse performance gate"]
fn optimization_batch_20260826da_runtime144_mobility_parse_performance_evidence() {
    const VALUE_COUNT: usize = 16_384;
    const ITERATIONS_PER_SAMPLE: usize = 16;
    const SAMPLE_COUNT: usize = 17;
    const PARSE_COUNT: usize = VALUE_COUNT * ITERATIONS_PER_SAMPLE;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME144_MOBILITY_BORROWED_ENUM_PARSE_BENCH_V1"
    );
    let values = (0..VALUE_COUNT)
        .map(|index| {
            if index % 2 == 0 {
                " DyNaMiC ".to_string()
            } else {
                " dynamic ".to_string()
            }
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(parse_batch(
            &values,
            ITERATIONS_PER_SAMPLE,
            legacy_parse_mobility,
        ));
        black_box(parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_mobility));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, legacy_parse_mobility)
            }));
            optimized_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_mobility)
            }));
        } else {
            optimized_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_mobility)
            }));
            legacy_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, legacy_parse_mobility)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} values={VALUE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} parses={PARSE_COUNT} samples={SAMPLE_COUNT} legacy_keyword_allocations={PARSE_COUNT} optimized_keyword_allocations=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed mobility parse P95 {optimized_p95_ns}ns must be at most 70% of lowercase-allocation P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_mobility(value: &str) -> Option<Mobility> {
    match value.trim().to_ascii_lowercase().as_str() {
        "dynamic" => Some(Mobility::Dynamic),
        "static" => Some(Mobility::Static),
        _ => None,
    }
}

fn parse_batch(values: &[String], iterations: usize, parse: fn(&str) -> Option<Mobility>) -> usize {
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
