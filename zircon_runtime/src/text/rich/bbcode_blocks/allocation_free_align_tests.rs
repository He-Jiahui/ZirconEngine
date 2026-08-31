use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::text::TextAlign;

use super::parse_align;

const PERFORMANCE_MARKER: &str = "RUNTIME143_BBCODE_ALIGN_ZERO_ALLOCATION_PARSE_BENCH_V1";

#[test]
fn optimization_batch_20260826cz_runtime143_bbcode_align_preserves_case_insensitive_keywords() {
    for (value, expected) in [
        (" LEFT ", Some(TextAlign::Left)),
        ("CeNtEr", Some(TextAlign::Center)),
        ("RIGHT", Some(TextAlign::Right)),
        ("fill", Some(TextAlign::Justify)),
        ("JuStIfY", Some(TextAlign::Justify)),
        ("START", Some(TextAlign::Start)),
        ("end", Some(TextAlign::End)),
        ("middle", None),
    ] {
        assert_eq!(parse_align(value), expected, "{value}");
    }
}

#[test]
fn optimization_batch_20260826cz_runtime143_bbcode_align_avoids_owned_lowercase() {
    let source = include_str!("../bbcode_blocks.rs");
    let parser = source
        .split_once("fn parse_align")
        .expect("align parser should exist")
        .1
        .split_once("fn parse_indent")
        .expect("indent parser should follow align parser")
        .0;

    assert!(parser.contains("eq_ignore_ascii_case"));
    assert!(!parser.contains("to_ascii_lowercase()"));
}

#[test]
#[ignore = "release-only BBCode alignment performance gate"]
fn optimization_batch_20260826cz_runtime143_bbcode_align_performance_evidence() {
    const VALUE_COUNT: usize = 16_384;
    const ITERATIONS_PER_SAMPLE: usize = 16;
    const SAMPLE_COUNT: usize = 17;
    const PARSE_COUNT: usize = VALUE_COUNT * ITERATIONS_PER_SAMPLE;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME143_BBCODE_ALIGN_ZERO_ALLOCATION_PARSE_BENCH_V1"
    );
    let values = (0..VALUE_COUNT)
        .map(|index| {
            if index % 2 == 0 {
                " LEFT ".to_string()
            } else {
                " left ".to_string()
            }
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(parse_batch(
            &values,
            ITERATIONS_PER_SAMPLE,
            legacy_parse_align,
        ));
        black_box(parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_align));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, legacy_parse_align)
            }));
            optimized_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_align)
            }));
        } else {
            optimized_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, parse_align)
            }));
            legacy_samples.push(measure(|| {
                parse_batch(&values, ITERATIONS_PER_SAMPLE, legacy_parse_align)
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
        "allocation-free BBCode align P95 {optimized_p95_ns}ns must be at most 70% of lowercase-allocation P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_align(value: &str) -> Option<TextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" => Some(TextAlign::Left),
        "center" => Some(TextAlign::Center),
        "right" => Some(TextAlign::Right),
        "fill" | "justify" => Some(TextAlign::Justify),
        "start" => Some(TextAlign::Start),
        "end" => Some(TextAlign::End),
        _ => None,
    }
}

fn parse_batch(
    values: &[String],
    iterations: usize,
    parse: fn(&str) -> Option<TextAlign>,
) -> usize {
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
