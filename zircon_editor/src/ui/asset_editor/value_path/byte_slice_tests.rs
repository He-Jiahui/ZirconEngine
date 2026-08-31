use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_MARKER: &str = "EDITOR23_VALUE_PATH_BYTE_SLICE_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const PARSES_PER_SAMPLE: usize = 64;
const SEGMENT_PAIRS: usize = 1_024;

fn legacy_parse_value_path(path: &str) -> Option<Vec<UiAssetTomlPathSegment>> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut segments = Vec::new();
    let chars = trimmed.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        match chars[index] {
            '.' => index += 1,
            '[' => {
                index += 1;
                let start = index;
                while index < chars.len() && chars[index] != ']' {
                    index += 1;
                }
                if index == start || index >= chars.len() {
                    return None;
                }
                let value = chars[start..index].iter().collect::<String>();
                let parsed = value.trim().parse::<usize>().ok()?;
                segments.push(UiAssetTomlPathSegment::Index(parsed));
                index += 1;
            }
            _ => {
                let start = index;
                while index < chars.len() && chars[index] != '.' && chars[index] != '[' {
                    index += 1;
                }
                let value = chars[start..index].iter().collect::<String>();
                let value = value.trim();
                if value.is_empty() {
                    return None;
                }
                segments.push(UiAssetTomlPathSegment::Key(value.to_string()));
            }
        }
    }

    (!segments.is_empty()).then_some(segments)
}

fn deep_unicode_path() -> String {
    let mut path = String::with_capacity(SEGMENT_PAIRS * 24);
    for index in 0..SEGMENT_PAIRS {
        if index > 0 {
            path.push('.');
        }
        path.push_str("node_");
        path.push_str(&index.to_string());
        path.push_str("组件");
        path.push('[');
        path.push_str(&(index % 17).to_string());
        path.push(']');
    }
    path
}

fn sample_ns(path: &str, mut parse: impl FnMut(&str) -> usize) -> u128 {
    let started = Instant::now();
    let mut observed = 0usize;
    for _ in 0..PARSES_PER_SAMPLE {
        observed += black_box(parse(black_box(path)));
    }
    black_box(observed);
    started.elapsed().as_nanos()
}

fn percentile(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826ba_value_path_byte_slices_preserve_parser_semantics() {
    for path in [
        " root.节点[ 12 ].leaf ",
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
            "parser behavior changed for {path:?}"
        );
    }
}

#[test]
fn optimization_batch_20260826ba_value_path_parser_uses_utf8_byte_slices() {
    let source = include_str!("../value_path.rs");

    assert!(source.contains("let bytes = trimmed.as_bytes();"));
    assert!(source.contains("trimmed[start..index].trim()"));
    assert!(!source.contains("trimmed.chars().collect::<Vec<_>>()"));
    assert!(!source.contains("chars[start..index].iter().collect::<String>()"));
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260826ba_value_path_byte_slice_p95() {
    let path = deep_unicode_path();
    assert_eq!(parse_value_path(&path), legacy_parse_value_path(&path));
    for _ in 0..8 {
        black_box(legacy_parse_value_path(&path));
        black_box(parse_value_path(&path));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(&path, |path| {
                legacy_parse_value_path(path).expect("legacy parse").len()
            }));
            optimized_samples.push(sample_ns(&path, |path| {
                parse_value_path(path).expect("optimized parse").len()
            }));
        } else {
            optimized_samples.push(sample_ns(&path, |path| {
                parse_value_path(path).expect("optimized parse").len()
            }));
            legacy_samples.push(sample_ns(&path, |path| {
                legacy_parse_value_path(path).expect("legacy parse").len()
            }));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples.clone(), 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction = 100.0 - (optimized_p95 as f64 * 100.0 / legacy_p95 as f64);
    println!(
        "{BENCHMARK_MARKER} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} reduction_percent={reduction:.3} segment_pairs={SEGMENT_PAIRS} parses_per_sample={PARSES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS}"
    );

    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "expected byte-slice parser P95 to be at least 30% below char-vector parsing; legacy={legacy_p95}ns optimized={optimized_p95}ns reduction={reduction:.3}%"
    );
}
