use std::hint::black_box;
use std::time::Instant;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::text::UiRichLinkTarget;

use crate::text::{LinkRef, StyleOverride, StyledRun};

use super::*;

const ASCII_BYTE_COUNT: usize = 250_000;
const ASCII_RUN_COUNT: usize = 128;
const UNICODE_GRAPHEME_COUNT: usize = 50_000;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimized_grapheme_alignment_matches_legacy_normalization() {
    let ascii = "abcdefgh".repeat(32);
    let canonical_ascii_runs = ascii_runs(ascii.len(), 8);
    assert_alignment_matches(&ascii, &canonical_ascii_runs);

    let gap_runs = vec![fixture_run((1, 3), 500, "gap")];
    assert_alignment_matches("abcd", &gap_runs);

    let adjacent_equal_runs = vec![
        fixture_run((0, 2), 500, "same"),
        fixture_run((2, 4), 500, "same"),
    ];
    assert_alignment_matches("abcd", &adjacent_equal_runs);

    let combining = "e\u{301}x";
    let split_combining_runs = vec![
        fixture_run((0, 1), 700, "first"),
        fixture_run((1, combining.len() as u32), 400, "second"),
    ];
    assert_alignment_matches(combining, &split_combining_runs);

    assert_alignment_matches("", &[]);
}

#[test]
#[ignore = "release-only grapheme alignment benchmark"]
fn grapheme_alignment_metadata_clone_release_benchmark_evidence() {
    let ascii = "a".repeat(ASCII_BYTE_COUNT);
    let ascii_runs = ascii_runs(ASCII_BYTE_COUNT, ASCII_RUN_COUNT);
    let unicode = "e\u{301}".repeat(UNICODE_GRAPHEME_COUNT);
    let unicode_runs = vec![fixture_run(
        (0, u32::try_from(unicode.len()).unwrap()),
        700,
        &"unicode-link".repeat(16),
    )];

    black_box(time_legacy(&ascii, &ascii_runs));
    black_box(time_optimized(&ascii, &ascii_runs));
    black_box(time_legacy(&unicode, &unicode_runs));
    black_box(time_optimized(&unicode, &unicode_runs));

    let mut legacy_ascii_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ascii_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_unicode_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_unicode_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ascii_samples.push(time_legacy(&ascii, &ascii_runs));
            optimized_ascii_samples.push(time_optimized(&ascii, &ascii_runs));
            legacy_unicode_samples.push(time_legacy(&unicode, &unicode_runs));
            optimized_unicode_samples.push(time_optimized(&unicode, &unicode_runs));
        } else {
            optimized_ascii_samples.push(time_optimized(&ascii, &ascii_runs));
            legacy_ascii_samples.push(time_legacy(&ascii, &ascii_runs));
            optimized_unicode_samples.push(time_optimized(&unicode, &unicode_runs));
            legacy_unicode_samples.push(time_legacy(&unicode, &unicode_runs));
        }
    }

    let legacy_ascii_p95_ns = nearest_rank(&legacy_ascii_samples, 95);
    let optimized_ascii_p95_ns = nearest_rank(&optimized_ascii_samples, 95);
    let legacy_unicode_p95_ns = nearest_rank(&legacy_unicode_samples, 95);
    let optimized_unicode_p95_ns = nearest_rank(&optimized_unicode_samples, 95);

    println!(
        "RUNTIME84_GRAPHEME_ALIGNMENT_PERF ascii_bytes=250000 ascii_runs=128 unicode_graphemes=50000 pairs=21 order=alternating percentile=nearest-rank legacy_ascii_metadata_clones=250000 optimized_ascii_metadata_clones=128 legacy_unicode_metadata_clones=50000 optimized_unicode_metadata_clones=1 legacy_ascii_p50_ns={} legacy_ascii_p95_ns={} optimized_ascii_p50_ns={} optimized_ascii_p95_ns={} legacy_unicode_p50_ns={} legacy_unicode_p95_ns={} optimized_unicode_p50_ns={} optimized_unicode_p95_ns={} legacy_ascii_samples_ns={:?} optimized_ascii_samples_ns={:?} legacy_unicode_samples_ns={:?} optimized_unicode_samples_ns={:?}",
        nearest_rank(&legacy_ascii_samples, 50),
        legacy_ascii_p95_ns,
        nearest_rank(&optimized_ascii_samples, 50),
        optimized_ascii_p95_ns,
        nearest_rank(&legacy_unicode_samples, 50),
        legacy_unicode_p95_ns,
        nearest_rank(&optimized_unicode_samples, 50),
        optimized_unicode_p95_ns,
        legacy_ascii_samples,
        optimized_ascii_samples,
        legacy_unicode_samples,
        optimized_unicode_samples,
    );

    assert!(
        optimized_ascii_p95_ns.saturating_mul(4) <= legacy_ascii_p95_ns,
        "canonical ASCII fast path must reduce P95 by at least 75%: legacy={legacy_ascii_p95_ns}ns optimized={optimized_ascii_p95_ns}ns"
    );
    assert!(
        optimized_unicode_p95_ns.saturating_mul(4) <= legacy_unicode_p95_ns,
        "borrowed Unicode metadata must reduce P95 by at least 75%: legacy={legacy_unicode_p95_ns}ns optimized={optimized_unicode_p95_ns}ns"
    );
}

fn ascii_runs(text_len: usize, run_count: usize) -> Vec<StyledRun> {
    (0..run_count)
        .map(|index| {
            let start = index * text_len / run_count;
            let end = (index + 1) * text_len / run_count;
            fixture_run(
                (u32::try_from(start).unwrap(), u32::try_from(end).unwrap()),
                if index % 2 == 0 { 400 } else { 700 },
                &format!(
                    "res://docs/grapheme-alignment/{index:05}/{}",
                    "x".repeat(96)
                ),
            )
        })
        .collect()
}

fn fixture_run(byte_range: (u32, u32), weight: u16, href: &str) -> StyledRun {
    StyledRun {
        byte_range,
        style: StyleOverride {
            weight: Some(weight),
            ..StyleOverride::default()
        },
        inline: None,
        link: Some(LinkRef {
            target: UiRichLinkTarget::parse(href).expect("fixture link is engine-local"),
            tooltip: None,
        }),
    }
}

fn assert_alignment_matches(text: &str, runs: &[StyledRun]) {
    assert_eq!(
        align_runs_to_graphemes(text, runs),
        legacy_align_runs_to_graphemes(text, runs)
    );
}

fn legacy_align_runs_to_graphemes(text: &str, runs: &[StyledRun]) -> Vec<StyledRun> {
    let mut aligned = Vec::new();
    let mut run_index = 0;
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        while run_index < runs.len()
            && usize::try_from(runs[run_index].byte_range.1).unwrap_or(0) <= start
        {
            run_index += 1;
        }
        let source = runs
            .get(run_index)
            .filter(|run| range_contains(run.byte_range, start))
            .cloned()
            .unwrap_or_default();
        let mut run = styled_run(start, end, source.style);
        run.inline = source.inline;
        run.link = source.link;
        push_or_merge_run(&mut aligned, run);
    }
    aligned
}

fn time_legacy(text: &str, runs: &[StyledRun]) -> u128 {
    let started = Instant::now();
    let aligned = legacy_align_runs_to_graphemes(black_box(text), black_box(runs));
    let elapsed = started.elapsed().as_nanos();
    black_box(aligned);
    elapsed
}

fn time_optimized(text: &str, runs: &[StyledRun]) -> u128 {
    let started = Instant::now();
    let aligned = align_runs_to_graphemes(black_box(text), black_box(runs));
    let elapsed = started.elapsed().as_nanos();
    black_box(aligned);
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
