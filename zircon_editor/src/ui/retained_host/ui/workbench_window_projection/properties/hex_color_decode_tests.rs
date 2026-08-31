use std::hint::black_box;
use std::time::Instant;

use super::*;

const COLOR_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 21;
const COLORS: [&str; 8] = [
    "#000000",
    "#ffffff",
    "#1976d2",
    "#A1B2C3",
    "#01020304",
    "#89abcdef",
    "#FFEEDDCC",
    "#13579bdf",
];

#[test]
fn optimization_batch_20260826cm_editor76_hex_color_decode_preserves_rgba_contract() {
    assert_eq!(parse_hex_rgba("#1976d2"), Some([0x19, 0x76, 0xd2, 0xff]));
    assert_eq!(
        parse_hex_rgba("  #A1b2C3d4  "),
        Some([0xa1, 0xb2, 0xc3, 0xd4])
    );
    assert_eq!(parse_hex_rgba("#xyzxyz"), None);
    assert_eq!(parse_hex_rgba("#12345"), None);
    assert_eq!(parse_hex_rgba("#1\u{e9}234"), None);
}

#[test]
fn optimization_batch_20260826cm_editor76_hex_color_decode_uses_ascii_nibbles() {
    let source = include_str!("../properties.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("fn hex_nibble("));
    assert!(source.contains("strip_prefix('#')?.as_bytes()"));
    assert!(!source.contains("from_str_radix"));
    assert!(!source.contains("&hex[range]"));
}

fn legacy_parse(raw: &str) -> Option<[u8; 4]> {
    let hex = raw.trim().strip_prefix('#')?;
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match hex.len() {
        6 => Some([channel(0..2)?, channel(2..4)?, channel(4..6)?, 255]),
        8 => Some([
            channel(0..2)?,
            channel(2..4)?,
            channel(4..6)?,
            channel(6..8)?,
        ]),
        _ => None,
    }
}

fn decode_batch(parser: fn(&str) -> Option<[u8; 4]>) -> u64 {
    let mut checksum = 0u64;
    for index in 0..COLOR_COUNT {
        let rgba = parser(black_box(COLORS[index % COLORS.len()])).unwrap();
        checksum = checksum.wrapping_add(u64::from(rgba[index & 3]));
    }
    checksum
}

fn elapsed_ns(run: impl FnOnce() -> u64) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826cm_editor76_hex_color_decode_performance_evidence() {
    for _ in 0..3 {
        assert_eq!(
            black_box(decode_batch(legacy_parse)),
            decode_batch(parse_hex_rgba)
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(|| decode_batch(legacy_parse)));
            optimized_samples.push(elapsed_ns(|| decode_batch(parse_hex_rgba)));
        } else {
            optimized_samples.push(elapsed_ns(|| decode_batch(parse_hex_rgba)));
            legacy_samples.push(elapsed_ns(|| decode_batch(legacy_parse)));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "EDITOR76_HEX_COLOR_NIBBLE_DECODE_BENCH_V1 sample_pairs={} color_count={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        COLOR_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "ASCII nibble decoding p95 must be at least 30% below generic radix parsing: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
