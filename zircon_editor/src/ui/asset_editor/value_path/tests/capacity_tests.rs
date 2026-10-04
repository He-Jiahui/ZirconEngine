use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_MARKER: &str = "EDITOR23_VALUE_PATH_CAPACITY_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const SEGMENT_PAIRS: usize = 2_048;
const PARSES_PER_SAMPLE: usize = 32;

#[test]
fn optimization_batch_20260915_value_path_capacity_preserves_semantics() {
    for path in [
        "root.节点[ 12 ].leaf",
        ".foo..bar.",
        "[0]name",
        "foo]",
        "bad[]",
        "bad[",
        "   ",
    ] {
        assert_eq!(
            parse_value_path(path),
            legacy_parse_value_path(path),
            "{path:?}"
        );
    }

    let parsed = parse_value_path("root[1].child[2]").expect("valid value path");
    assert_eq!(parsed.len(), 4);
    assert!(parsed.capacity() >= parsed.len());

    let delimiters = ".".repeat(4096);
    assert!(parse_value_path(&delimiters).is_none());
}

#[test]
fn optimization_batch_20260915_value_path_capacity_reserves_only_after_a_value() {
    let source = include_str!("../../value_path.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("value path production source");
    let helper = production
        .split("fn reserve_first_segment_capacity")
        .nth(1)
        .expect("first-segment capacity helper");
    assert!(helper.contains("if !segments.is_empty()"));
    assert!(helper.contains("segments.reserve(capacity)"));
    assert!(helper.contains(".iter()"));
    assert!(helper.contains(".filter"));
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260915_value_path_capacity_p95() {
    let path = deep_value_path();
    assert_eq!(parse_value_path(&path), legacy_parse_value_path(&path));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(&path, legacy_parse_value_path));
            optimized_samples.push(sample_ns(&path, parse_value_path));
        } else {
            optimized_samples.push(sample_ns(&path, parse_value_path));
            legacy_samples.push(sample_ns(&path, legacy_parse_value_path));
        }
    }

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_growth_events = geometric_growth_events(SEGMENT_PAIRS * 2);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{BENCHMARK_MARKER} segment_pairs={SEGMENT_PAIRS} parses_per_sample={PARSES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
}

fn legacy_parse_value_path(path: &str) -> Option<Vec<UiAssetTomlPathSegment>> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut segments = Vec::new();
    let bytes = trimmed.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'.' => index += 1,
            b'[' => {
                index += 1;
                let start = index;
                while index < bytes.len() && bytes[index] != b']' {
                    index += 1;
                }
                if index == start || index >= bytes.len() {
                    return None;
                }
                let parsed = trimmed[start..index].trim().parse::<usize>().ok()?;
                segments.push(UiAssetTomlPathSegment::Index(parsed));
                index += 1;
            }
            _ => {
                let start = index;
                while index < bytes.len() && bytes[index] != b'.' && bytes[index] != b'[' {
                    index += 1;
                }
                let value = trimmed[start..index].trim();
                if value.is_empty() {
                    return None;
                }
                segments.push(UiAssetTomlPathSegment::Key(value.to_string()));
            }
        }
    }

    (!segments.is_empty()).then_some(segments)
}

fn deep_value_path() -> String {
    let mut path = String::with_capacity(SEGMENT_PAIRS * 18);
    for index in 0..SEGMENT_PAIRS {
        if index > 0 {
            path.push('.');
        }
        path.push_str("node");
        path.push_str(&index.to_string());
        path.push('[');
        path.push_str(&(index % 17).to_string());
        path.push(']');
    }
    path
}

fn sample_ns(
    path: &str,
    mut parse: impl FnMut(&str) -> Option<Vec<UiAssetTomlPathSegment>>,
) -> u128 {
    let started = Instant::now();
    let mut observed = 0usize;
    for _ in 0..PARSES_PER_SAMPLE {
        observed += black_box(parse(black_box(path)).expect("benchmark path").len());
    }
    black_box(observed);
    started.elapsed().as_nanos().max(1)
}

fn geometric_growth_events(segment_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut growth_events = 0usize;
    for length in 1..=segment_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            growth_events += 1;
        }
    }
    growth_events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
