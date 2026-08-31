use std::hint::black_box;
use std::time::Instant;

use super::{generic_font_family, is_system_ui_family};

const PERF_MARKER: &str = "EDITOR80_FONT_FAMILY_ZERO_ALLOCATION_MATCH_BENCH_V1";

#[test]
fn optimization_batch_20260826cq_editor_font_family_match_preserves_aliases() {
    for family in ["system-ui", " SYSTEM-UI ", "Ui-Sans-Serif"] {
        assert!(is_system_ui_family(family));
    }
    for family in [
        "sans-serif",
        " SANS-SERIF ",
        "Monospace",
        "UI-MONOSPACE",
        "Serif",
        "CURSIVE",
        "fantasy",
    ] {
        assert!(generic_font_family(family).is_some(), "family={family}");
    }
    assert!(!is_system_ui_family("system-ui-é"));
    assert!(generic_font_family("sans-sérif").is_none());
}

#[test]
fn optimization_batch_20260826cq_editor_font_family_match_source_contract() {
    let source = include_str!("../font.rs");

    assert!(source.contains("family.eq_ignore_ascii_case(\"system-ui\")"));
    assert!(source.contains("family.eq_ignore_ascii_case(\"sans-serif\")"));
    assert!(!source.contains("family.trim().to_ascii_lowercase()"));
    assert_eq!(
        PERF_MARKER,
        "EDITOR80_FONT_FAMILY_ZERO_ALLOCATION_MATCH_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826cq_editor_font_family_match_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 120_000;
    let families = [
        " SYSTEM-UI ",
        "ui-sans-serif",
        "SANS-SERIF",
        " Monospace ",
        "UI-MONOSPACE",
        "custom-editor-face",
    ];

    black_box(measure_legacy(&families, LOOKUPS_PER_SAMPLE / 10));
    black_box(measure_optimized(&families, LOOKUPS_PER_SAMPLE / 10));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&families, LOOKUPS_PER_SAMPLE));
            optimized_ns.push(measure_optimized(&families, LOOKUPS_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_optimized(&families, LOOKUPS_PER_SAMPLE));
            legacy_ns.push(measure_legacy(&families, LOOKUPS_PER_SAMPLE));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} aliases=6 order=alternating_legacy_first_even legacy_normalized_string_allocations_per_sample=200000 optimized_normalized_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "borrowed ASCII family matching must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_family_kind(family: &str) -> usize {
    if matches!(
        family.trim().to_ascii_lowercase().as_str(),
        "system-ui" | "ui-sans-serif"
    ) {
        return 1;
    }

    match family.trim().to_ascii_lowercase().as_str() {
        "sans-serif" | "monospace" | "ui-monospace" | "serif" | "cursive" | "fantasy" => 2,
        _ => 0,
    }
}

fn optimized_family_kind(family: &str) -> usize {
    if is_system_ui_family(family) {
        1
    } else if generic_font_family(family).is_some() {
        2
    } else {
        0
    }
}

fn measure_legacy(families: &[&str], lookups: usize) -> u128 {
    measure(families, lookups, legacy_family_kind)
}

fn measure_optimized(families: &[&str], lookups: usize) -> u128 {
    measure(families, lookups, optimized_family_kind)
}

fn measure(families: &[&str], lookups: usize, classify: fn(&str) -> usize) -> u128 {
    let mut checksum = 0usize;
    let started = Instant::now();
    for index in 0..lookups {
        checksum = checksum.wrapping_add(classify(black_box(families[index % families.len()])));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
