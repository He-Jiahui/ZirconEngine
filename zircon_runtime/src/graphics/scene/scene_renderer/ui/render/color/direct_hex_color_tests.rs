use std::hint::black_box;
use std::time::Instant;

use super::parse_hex_color;

const SAMPLE_PAIRS: usize = 21;
const PARSES_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826ej_runtime179_scene_ui_hex_preserves_opacity() {
    assert_eq!(
        parse_hex_color("#12aBcF", 0.5),
        Some([
            0x12 as f32 / 255.0,
            0xab as f32 / 255.0,
            0xcf as f32 / 255.0,
            0.5,
        ])
    );
    assert_eq!(
        parse_hex_color("#12ABCf80", 0.5),
        Some([
            0x12 as f32 / 255.0,
            0xab as f32 / 255.0,
            0xcf as f32 / 255.0,
            (0x80 as f32 / 255.0) * 0.5,
        ])
    );
    for invalid in ["12abcf", "#fff", "#12abcg", "#12abcf8000", "#12ab\u{e7}f"] {
        assert_eq!(
            parse_hex_color(invalid, 1.0),
            None,
            "{invalid} must stay invalid"
        );
    }
}

#[test]
fn optimization_batch_20260826ej_runtime179_scene_ui_hex_uses_direct_byte_decoder() {
    let source = include_str!("../color.rs");
    let parser_start = source.find("pub(super) fn parse_hex_color").unwrap();
    let parser_end = source[parser_start..]
        .find("#[cfg(test)]")
        .map(|offset| parser_start + offset)
        .unwrap();
    let parser_source = &source[parser_start..parser_end];
    assert!(parser_source.contains("decode_hex_byte"));
    assert!(!parser_source.contains("from_str_radix"));
    assert!(!parser_source.contains("&hex["));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ej_runtime179_scene_ui_direct_hex_color_bench() {
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
        "RUNTIME179_SCENE_UI_DIRECT_HEX_COLOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} legacy_radix_calls=4 optimized_byte_decodes=4 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct scene UI hex decoding P95 {optimized_p95_ns}ns must be at most 70% of general radix decoding P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_hex_color(value: &str, opacity: f32) -> Option<[f32; 4]> {
    let hex = value.strip_prefix('#')?;
    let parse_byte = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match hex.len() {
        6 => {
            let red = parse_byte(0..2)?;
            let green = parse_byte(2..4)?;
            let blue = parse_byte(4..6)?;
            Some([
                red as f32 / 255.0,
                green as f32 / 255.0,
                blue as f32 / 255.0,
                opacity,
            ])
        }
        8 => {
            let red = parse_byte(0..2)?;
            let green = parse_byte(2..4)?;
            let blue = parse_byte(4..6)?;
            let alpha = parse_byte(6..8)?;
            Some([
                red as f32 / 255.0,
                green as f32 / 255.0,
                blue as f32 / 255.0,
                (alpha as f32 / 255.0) * opacity,
            ])
        }
        _ => None,
    }
}

fn measure_legacy(color: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u32;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(legacy_parse_hex_color(black_box(color), 0.75)).unwrap()[3].to_bits();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(color: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u32;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(parse_hex_color(black_box(color), 0.75)).unwrap()[3].to_bits();
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
