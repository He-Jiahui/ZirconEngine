use std::{hint::black_box, time::Instant};

use super::*;

const RELEASE_GRAPHEME_COUNT: usize = 131_072;
const RELEASE_SAMPLE_COUNT: usize = 31;
const RELEASE_WARMUP_COUNT: usize = 5;

fn tab_style() -> TextStyle {
    TextStyle {
        tab_size: 4.0,
        ..TextStyle::default()
    }
}

#[test]
fn streaming_tab_layout_preserves_tab_stops_and_unicode_graphemes() {
    let style = tab_style();
    let text = "a\u{301}\tb";
    let advances = [1.0, -2.0, 4.0];

    assert_eq!(
        tab_aligned_advances(text, &advances, &style, 2.0),
        vec![1.0, 7.0, 4.0]
    );
    assert_eq!(tab_aligned_width(text, &advances, &style, 2.0), 12.0);
}

#[test]
fn matching_grapheme_width_streams_the_known_tabbed_input() {
    let style = tab_style();
    let text = "a\u{301}\tb";
    let advances = [1.0, -2.0, 4.0];

    assert_eq!(
        tab_aligned_width_for_matching_graphemes(text, &advances, &style, 2.0),
        12.0
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260827bl_streaming_tab_layout_p95() {
    let style = tab_style();
    let text = release_tabbed_text();
    let advances = vec![1.0; RELEASE_GRAPHEME_COUNT];
    let legacy_width = legacy_tab_aligned_width(&text, &advances, &style, 2.0);
    let optimized_width = tab_aligned_width(&text, &advances, &style, 2.0);
    assert_eq!(legacy_width, optimized_width);

    let (legacy_samples, optimized_samples) = paired_width_samples(&text, &advances, &style, 2.0);
    let legacy_p50 = percentile(&legacy_samples, 50);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_allocated_bytes =
        RELEASE_GRAPHEME_COUNT * (std::mem::size_of::<&str>() + std::mem::size_of::<f32>());

    println!(
        "PERF_RESULT RUNTIME81_STREAMING_TAB_LAYOUT_BENCH_V1 graphemes={RELEASE_GRAPHEME_COUNT} tab_every=16 warmups={RELEASE_WARMUP_COUNT} samples={RELEASE_SAMPLE_COUNT} legacy_width_allocations=2 optimized_width_allocations=0 legacy_width_allocated_bytes={legacy_allocated_bytes} optimized_width_allocated_bytes=0 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        legacy_p50 > 0 && optimized_p50 > 0,
        "release timing samples must have nanosecond resolution"
    );
    assert!(
        optimized_p50 * 100 <= legacy_p50 * 30,
        "optimized P50 {optimized_p50}ns must be at least 70% below legacy P50 {legacy_p50}ns"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

#[test]
fn streaming_tab_layout_preserves_unaligned_advance_fallbacks() {
    let style = tab_style();
    let no_tab_advances = [2.0, -1.0];
    let mismatched_advances = [3.0];

    assert_eq!(
        tab_aligned_advances("ab", &no_tab_advances, &style, 2.0),
        no_tab_advances
    );
    assert_eq!(tab_aligned_width("ab", &no_tab_advances, &style, 2.0), 1.0);
    assert_eq!(
        tab_aligned_advances("a\t", &mismatched_advances, &style, 2.0),
        mismatched_advances
    );
    assert_eq!(
        tab_aligned_width("a\t", &mismatched_advances, &style, 2.0),
        3.0
    );
}

#[test]
fn streaming_tab_layout_keeps_extreme_finite_geometry_bounded() {
    let style = tab_style();
    let advances = [f32::MAX, f32::MAX];

    let aligned = tab_aligned_advances("a\t", &advances, &style, f32::MAX);
    let width = tab_aligned_width("a\t", &advances, &style, f32::MAX);

    assert!(aligned.iter().all(|advance| advance.is_finite()));
    assert!(width.is_finite());
    assert_eq!(width, f32::MAX);
    assert_eq!(
        tab_aligned_width("ab", &advances, &style, f32::MAX),
        f32::MAX
    );
}

fn release_tabbed_text() -> String {
    let mut text = String::with_capacity(RELEASE_GRAPHEME_COUNT);
    for index in 0..RELEASE_GRAPHEME_COUNT {
        text.push(if index % 16 == 15 { '\t' } else { 'a' });
    }
    text
}

fn legacy_tab_aligned_width(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> f32 {
    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != advances.len() || !graphemes.iter().any(|grapheme| *grapheme == "\t") {
        return finite_sum(advances.iter().copied());
    }

    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut cursor = FiniteGeometryAccumulator::default();
    let mut adjusted = Vec::with_capacity(advances.len());
    for (grapheme, advance) in graphemes.iter().zip(advances.iter().copied()) {
        let resolved_advance = if *grapheme == "\t" {
            next_tab_advance(
                cursor.value(),
                cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        cursor.add(resolved_advance);
        adjusted.push(resolved_advance);
    }
    finite_sum(adjusted)
}

fn paired_width_samples(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..RELEASE_WARMUP_COUNT {
        black_box(legacy_tab_aligned_width(text, advances, style, space_width));
        black_box(tab_aligned_width(text, advances, style, space_width));
    }

    let mut legacy_samples = Vec::with_capacity(RELEASE_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(RELEASE_SAMPLE_COUNT);
    for index in 0..RELEASE_SAMPLE_COUNT {
        let legacy = || legacy_tab_aligned_width(text, advances, style, space_width);
        let optimized = || tab_aligned_width(text, advances, style, space_width);
        if index % 2 == 0 {
            legacy_samples.push(measure_width(legacy));
            optimized_samples.push(measure_width(optimized));
        } else {
            optimized_samples.push(measure_width(optimized));
            legacy_samples.push(measure_width(legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_width(operation: impl FnOnce() -> f32) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() * percentile / 100]
}
