use std::hint::black_box;
use std::time::Instant;

use super::keyboard_text_contains_control;

const BYTES_PER_TEXT: usize = 65_536;
const CHECKS_PER_SAMPLE: usize = 256;
const SAMPLE_PAIRS: usize = 17;

fn legacy_contains_control(text: &str) -> bool {
    text.chars().any(char::is_control)
}

fn measure(text: &str, optimized: bool) -> (u128, usize) {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for round in 0..CHECKS_PER_SAMPLE {
        let contains_control = if optimized {
            keyboard_text_contains_control(black_box(text))
        } else {
            legacy_contains_control(black_box(text))
        };
        checksum ^= usize::from(contains_control) << (round & 7);
    }
    (started.elapsed().as_nanos().max(1), black_box(checksum))
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn runtime82_batch_keyboard_payload_byte_control_scan_preserves_results() {
    for text in [
        "",
        "plain ASCII",
        "line\nfeed",
        "delete\u{7f}",
        "c1\u{85}",
        "\u{4f8b}\u{5b50}",
        "\u{1f642}",
    ] {
        assert_eq!(
            keyboard_text_contains_control(text),
            legacy_contains_control(text),
            "{text:?}"
        );
    }
}

#[test]
fn runtime82_batch_keyboard_payload_uses_ascii_bytes_with_unicode_fallback() {
    let source = include_str!("../payload.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("fn keyboard_text_contains_control(text: &str) -> bool"));
    assert!(production.contains("if text.is_ascii()"));
    assert!(production.contains("text.as_bytes().iter().any(u8::is_ascii_control)"));
    assert!(production.contains("text.chars().any(char::is_control)"));
    assert!(production.contains("keyboard_text_contains_control(text)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime82_batch_keyboard_payload_byte_control_scan_p95() {
    let mut text = String::with_capacity(BYTES_PER_TEXT);
    while text.len() < BYTES_PER_TEXT {
        text.push_str("Zircon text payload 0123456789 ");
    }
    text.truncate(BYTES_PER_TEXT);
    assert!(!keyboard_text_contains_control(&text));

    for _ in 0..3 {
        black_box(measure(&text, false));
        black_box(measure(&text, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut byte_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_checksum = 0_usize;
    let mut byte_checksum = 0_usize;
    for pair in 0..SAMPLE_PAIRS {
        let (legacy, byte) = if pair % 2 == 0 {
            (measure(&text, false), measure(&text, true))
        } else {
            let byte = measure(&text, true);
            let legacy = measure(&text, false);
            (legacy, byte)
        };
        legacy_samples.push(legacy.0);
        byte_samples.push(byte.0);
        legacy_checksum = legacy.1;
        byte_checksum = byte.1;
    }

    assert_eq!(legacy_checksum, byte_checksum);
    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let byte_p50_ns = nearest_rank(&byte_samples, 50);
    let byte_p95_ns = nearest_rank(&byte_samples, 95);
    println!(
        "RUNTIME82_KEYBOARD_PAYLOAD_BYTE_CONTROL_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
         bytes_per_text={BYTES_PER_TEXT} checks_per_sample={CHECKS_PER_SAMPLE} \
         pair_order=alternating_legacy_even legacy_first_pairs=9 byte_first_pairs=8 \
         legacy_unicode_scalar_scans={CHECKS_PER_SAMPLE} byte_ascii_scans={CHECKS_PER_SAMPLE} \
         unicode_fallback_changes=0 legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} byte_p50_ns={byte_p50_ns} byte_p95_ns={byte_p95_ns} \
         checksum={legacy_checksum}"
    );
    assert!(byte_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7));
}
