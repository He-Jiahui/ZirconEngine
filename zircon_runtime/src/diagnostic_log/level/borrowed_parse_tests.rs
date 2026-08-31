use std::hint::black_box;
use std::time::Instant;

use super::{DiagnosticLogFilter, DiagnosticLogLevel, DiagnosticLogLevelParseError};

const SAMPLE_PAIRS: usize = 21;
const PARSES_PER_SAMPLE: usize = 262_144;
const FILTERS: [&str; 12] = [
    "VERBOSE", "Trace", "DEBUG", "Info", "LOG", "Warning", "WARN", "Error", "ERR", "OFF", "None",
    "Quiet",
];

#[test]
fn optimization_batch_20260826dg_runtime150_diagnostic_level_preserves_aliases() {
    for (value, expected) in [
        (
            "  TrAcE  ",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose),
        ),
        (
            "DEBUG",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug),
        ),
        (
            "Info",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
        ),
        (
            "WARNING",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn),
        ),
        (
            "Err",
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Error),
        ),
        ("Quiet", DiagnosticLogFilter::Off),
    ] {
        assert_eq!(DiagnosticLogFilter::parse(value).unwrap(), expected);
    }
    assert_eq!(
        DiagnosticLogFilter::parse(" Chatty ").unwrap_err().value(),
        "Chatty"
    );
}

#[test]
fn optimization_batch_20260826dg_runtime150_diagnostic_level_avoids_lowercase_buffer() {
    let source = include_str!("../level.rs");

    assert!(source.contains("value.eq_ignore_ascii_case(\"verbose\")"));
    assert!(source.contains("value.eq_ignore_ascii_case(\"quiet\")"));
    assert!(!source.contains("let normalized = value.to_ascii_lowercase()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dg_runtime150_diagnostic_level_borrowed_parse_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_parse));
            optimized_samples.push(measure(DiagnosticLogFilter::parse));
        } else {
            optimized_samples.push(measure(DiagnosticLogFilter::parse));
            legacy_samples.push(measure(legacy_parse));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME150_DIAGNOSTIC_LEVEL_BORROWED_PARSE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} aliases={} \
legacy_lowercase_allocations_per_sample={PARSES_PER_SAMPLE} \
optimized_lowercase_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        FILTERS.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed log-level parse P95 {optimized_p95_ns}ns must be at most 70% of lowercase-buffer parse P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse(
    value: impl AsRef<str>,
) -> Result<DiagnosticLogFilter, DiagnosticLogLevelParseError> {
    let value = value.as_ref().trim();
    let normalized = value.to_ascii_lowercase();
    match normalized.as_str() {
        "verbose" | "trace" => Ok(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)),
        "debug" => Ok(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug)),
        "log" | "info" => Ok(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log)),
        "warn" | "warning" => Ok(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)),
        "error" | "err" => Ok(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Error)),
        "off" | "none" | "quiet" => Ok(DiagnosticLogFilter::Off),
        _ => Err(DiagnosticLogLevelParseError::new(value)),
    }
}

fn measure(
    parse: impl Fn(&str) -> Result<DiagnosticLogFilter, DiagnosticLogLevelParseError>,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for index in 0..PARSES_PER_SAMPLE {
        checksum ^= match black_box(parse(black_box(FILTERS[index % FILTERS.len()])).unwrap()) {
            DiagnosticLogFilter::Off => 0,
            DiagnosticLogFilter::Minimum(level) => level as usize + 1,
        };
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
