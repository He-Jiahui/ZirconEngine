use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::text::{InlineBaseline, StyleOverride};

use super::{apply_text_decoration, parse_font_weight, parse_inline_baseline};

const PERFORMANCE_MARKER: &str = "RUNTIME142_HTML_KEYWORD_ZERO_ALLOCATION_PARSE_BENCH_V1";

#[test]
fn optimization_batch_20260826cy_runtime142_html_keywords_preserve_case_insensitive_parsing() {
    assert_eq!(
        parse_inline_baseline("  BaSeLiNe  "),
        Some(InlineBaseline::Baseline)
    );
    assert_eq!(
        parse_inline_baseline("cEnTeR"),
        Some(InlineBaseline::Center)
    );
    assert_eq!(parse_inline_baseline("TOP"), Some(InlineBaseline::Top));
    assert_eq!(
        parse_inline_baseline("bottom"),
        Some(InlineBaseline::Bottom)
    );
    assert_eq!(parse_inline_baseline("middle"), None);

    assert_eq!(parse_font_weight(" NoRmAl "), Some(400));
    assert_eq!(parse_font_weight("BOLD"), Some(700));
    assert_eq!(parse_font_weight("650"), Some(650));
    assert_eq!(parse_font_weight("1001"), None);
}

#[test]
fn optimization_batch_20260826cy_runtime142_text_decoration_preserves_mixed_case_tokens() {
    let mut style = StyleOverride::default();
    apply_text_decoration("UnDeRlInE LINE-through", &mut style);

    assert_eq!(style.underline, Some(true));
    assert_eq!(style.strike, Some(true));

    apply_text_decoration("NoNe", &mut style);
    assert_eq!(style.underline, Some(false));
    assert_eq!(style.strike, Some(false));
}

#[test]
#[ignore = "release-only HTML keyword parsing performance gate"]
fn optimization_batch_20260826cy_runtime142_html_keyword_parse_performance_evidence() {
    const ATTRIBUTE_COUNT: usize = 16_384;
    const SAMPLE_COUNT: usize = 17;
    const LEGACY_ALLOCATIONS_PER_ATTRIBUTE: usize = 4;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME142_HTML_KEYWORD_ZERO_ALLOCATION_PARSE_BENCH_V1"
    );
    let attributes = (0..ATTRIBUTE_COUNT)
        .map(|index| {
            if index % 2 == 0 {
                (" CeNtEr ", " BoLd ", "UnDeRlInE LiNe-ThRoUgH")
            } else {
                (" BaSeLiNe ", " NoRmAl ", "LiNe-ThRoUgH UnDeRlInE")
            }
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(legacy_parse_batch(&attributes));
        black_box(optimized_parse_batch(&attributes));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| legacy_parse_batch(&attributes)));
            optimized_samples.push(measure(|| optimized_parse_batch(&attributes)));
        } else {
            optimized_samples.push(measure(|| optimized_parse_batch(&attributes)));
            legacy_samples.push(measure(|| legacy_parse_batch(&attributes)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let legacy_allocations = ATTRIBUTE_COUNT * LEGACY_ALLOCATIONS_PER_ATTRIBUTE;
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} attributes={ATTRIBUTE_COUNT} samples={SAMPLE_COUNT} legacy_keyword_allocations={legacy_allocations} optimized_keyword_allocations=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "allocation-free HTML keyword P95 {optimized_p95_ns}ns must be at most 70% of lowercase-allocation P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_batch(attributes: &[(&str, &str, &str)]) -> usize {
    attributes
        .iter()
        .map(|(baseline, weight, decoration)| {
            let baseline = legacy_parse_inline_baseline(black_box(baseline)).is_some() as usize;
            let weight = legacy_parse_font_weight(black_box(weight)).unwrap_or_default() as usize;
            let decoration = legacy_decoration_count(black_box(decoration));
            baseline + weight + decoration
        })
        .sum()
}

fn optimized_parse_batch(attributes: &[(&str, &str, &str)]) -> usize {
    attributes
        .iter()
        .map(|(baseline, weight, decoration)| {
            let baseline = parse_inline_baseline(black_box(baseline)).is_some() as usize;
            let weight = parse_font_weight(black_box(weight)).unwrap_or_default() as usize;
            let mut style = StyleOverride::default();
            apply_text_decoration(black_box(decoration), &mut style);
            baseline
                + weight
                + style.underline.unwrap_or_default() as usize
                + style.strike.unwrap_or_default() as usize
        })
        .sum()
}

fn legacy_parse_inline_baseline(value: &str) -> Option<InlineBaseline> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(InlineBaseline::Baseline),
        "center" => Some(InlineBaseline::Center),
        "top" => Some(InlineBaseline::Top),
        "bottom" => Some(InlineBaseline::Bottom),
        _ => None,
    }
}

fn legacy_parse_font_weight(value: &str) -> Option<u16> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(400),
        "bold" => Some(700),
        value => value
            .parse::<u16>()
            .ok()
            .filter(|weight| (1..=1000).contains(weight)),
    }
}

fn legacy_decoration_count(value: &str) -> usize {
    value
        .split_ascii_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| matches!(token.as_str(), "underline" | "line-through" | "none"))
        .count()
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
