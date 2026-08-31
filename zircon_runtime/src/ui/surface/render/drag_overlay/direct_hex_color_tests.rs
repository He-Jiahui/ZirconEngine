use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::style::UiRgbaColor;

use super::parse_css_color;

const SAMPLE_PAIRS: usize = 21;
const PARSES_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826ef_runtime175_hex_color_preserves_supported_forms() {
    assert_eq!(
        parse_css_color("  #12aBcF  ").map(UiRgbaColor::to_u8),
        Some([0x12, 0xab, 0xcf, 0xff])
    );
    assert_eq!(
        parse_css_color("#12ABCf80").map(UiRgbaColor::to_u8),
        Some([0x12, 0xab, 0xcf, 0x80])
    );
    for invalid in ["12abcf", "#fff", "#12abcg", "#12abcf8000", "#12ab\u{e7}f"] {
        assert_eq!(
            parse_css_color(invalid),
            None,
            "{invalid} must stay invalid"
        );
    }
}

#[test]
fn optimization_batch_20260826ef_runtime175_hex_color_uses_direct_byte_decoder() {
    let source = include_str!("../drag_overlay.rs");
    let parser_start = source.find("fn parse_css_color").unwrap();
    let parser_end = source[parser_start..]
        .find("fn css_color")
        .map(|offset| parser_start + offset)
        .unwrap();
    let parser_source = &source[parser_start..parser_end];
    assert!(parser_source.contains("decode_hex_byte"));
    assert!(!parser_source.contains("from_str_radix"));
    assert!(!parser_source.contains("is_ascii_hexdigit"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ef_runtime175_drag_overlay_direct_hex_color_bench() {
    let color = " #31A7d9c0 ";
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
        "RUNTIME175_DRAG_OVERLAY_DIRECT_HEX_COLOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} legacy_validation_passes=1 legacy_radix_calls=4 \
optimized_byte_passes=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct hex color decoding P95 {optimized_p95_ns}ns must be at most 70% of validated radix decoding P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_css_color(value: &str) -> Option<UiRgbaColor> {
    let encoded = value.trim().strip_prefix('#')?;
    if !encoded.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let (red, green, blue, alpha) = match encoded.len() {
        6 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::MAX,
        ),
        8 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::from_str_radix(&encoded[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some(UiRgbaColor::from_u8(red, green, blue, alpha))
}

fn measure_legacy(color: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u8;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(legacy_parse_css_color(black_box(color)))
            .unwrap()
            .to_u8()[3];
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(color: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u8;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(parse_css_color(black_box(color)))
            .unwrap()
            .to_u8()[3];
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
