use std::hint::black_box;
use std::time::Instant;

use crate::ui::retained_host::primitives::Color;

use super::parse_hex_color;

const SAMPLE_PAIRS: usize = 21;
const PARSES_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826ei_editor124_pane_hex_preserves_rgb_and_argb_order() {
    assert_eq!(
        parse_hex_color("#12aBcF"),
        Some(Color::from_argb_u8(0xff, 0x12, 0xab, 0xcf))
    );
    assert_eq!(
        parse_hex_color("#12ABCf80"),
        Some(Color::from_argb_u8(0x80, 0x12, 0xab, 0xcf))
    );
    for invalid in ["12abcf", "#fff", "#12abcg", "#12abcf8000", "#12ab\u{e7}f"] {
        assert_eq!(
            parse_hex_color(invalid),
            None,
            "{invalid} must stay invalid"
        );
    }
}

#[test]
fn optimization_batch_20260826ei_editor124_pane_hex_uses_direct_byte_decoder() {
    let source = include_str!("../pane_value_conversion.rs");
    let parser_start = source.find("fn parse_hex_color").unwrap();
    let parser_end = source[parser_start..]
        .find("pub(super) fn value_as_options")
        .map(|offset| parser_start + offset)
        .unwrap();
    let parser_source = &source[parser_start..parser_end];
    assert!(parser_source.contains("decode_hex_byte"));
    assert!(!parser_source.contains("from_str_radix"));
    assert!(!parser_source.contains("&hex["));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ei_editor124_pane_value_direct_hex_color_bench() {
    let color = "#31A7d9c0";
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(color));
            optimized_samples.push(measure_optimized(color));
        } else {
            optimized_samples.push(measure_optimized(color));
            legacy_samples.push(measure_legacy(color));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR124_PANE_VALUE_DIRECT_HEX_COLOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} legacy_radix_calls=4 optimized_byte_decodes=4 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct pane hex decoding P95 {optimized_p95_ns}ns must be at most 70% of general radix decoding P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;
    let parse_pair = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match hex.len() {
        6 => Some(Color::from_rgb_u8(
            parse_pair(0..2)?,
            parse_pair(2..4)?,
            parse_pair(4..6)?,
        )),
        8 => Some(Color::from_argb_u8(
            parse_pair(6..8)?,
            parse_pair(0..2)?,
            parse_pair(2..4)?,
            parse_pair(4..6)?,
        )),
        _ => None,
    }
}

fn measure_legacy(color: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u8;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(legacy_parse_hex_color(black_box(color)))
            .unwrap()
            .a;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(color: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u8;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(parse_hex_color(black_box(color))).unwrap().a;
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
