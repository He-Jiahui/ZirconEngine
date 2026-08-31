use std::hint::black_box;
use std::time::Instant;

use super::escape_line;

const SAMPLE_PAIRS: usize = 21;
const ESCAPES_PER_SAMPLE: usize = 256;
const SOURCE_REPEATS: usize = 1_024;

#[test]
fn optimization_batch_20260826gp_editor182_escape_preserves_specials_and_utf8() {
    assert_eq!(escape_line("plain text"), "plain text");
    assert_eq!(
        escape_line("left\\right\r\nworld"),
        "left\\\\right\\r\\nworld"
    );
    assert_eq!(
        escape_line("zircon-\u{4e16}\u{754c}"),
        "zircon-\u{4e16}\u{754c}"
    );
}

#[test]
fn optimization_batch_20260826gp_editor182_escape_is_single_allocation_pass() {
    let source = include_str!("../record.rs");

    assert!(source.contains("let mut escaped = String::with_capacity(escaped_capacity);"));
    assert!(source.contains("for character in value.chars()"));
    assert!(source.contains("'\\\\' => escaped.push_str(\"\\\\\\\\\")"));
    assert!(!source.contains(".replace('\\\\', \"\\\\\\\\\")"));
    assert!(!source.contains(".replace('\\r', \"\\\\r\")"));
    assert!(!source.contains(".replace('\\n', \"\\\\n\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gp_editor182_log_field_single_pass_escape_bench() {
    let value = "frame\\path\r\nzircon-\u{4e16}\u{754c};".repeat(SOURCE_REPEATS);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&value, false));
            optimized_samples.push(measure(&value, true));
        } else {
            optimized_samples.push(measure(&value, true));
            legacy_samples.push(measure(&value, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR182_LOG_FIELD_SINGLE_PASS_ESCAPE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
escapes_per_sample={ESCAPES_PER_SAMPLE} source_repeats={SOURCE_REPEATS} \
source_bytes={} legacy_string_allocations_per_escape=3 \
optimized_string_allocations_per_escape=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        value.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(value: &str, single_pass: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for escape in 0..ESCAPES_PER_SAMPLE {
        let escaped = if single_pass {
            escape_line(black_box(value))
        } else {
            black_box(value)
                .replace('\\', "\\\\")
                .replace('\r', "\\r")
                .replace('\n', "\\n")
        };
        checksum ^= black_box(escaped.len() ^ escaped.capacity() ^ escape);
        black_box(escaped);
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
