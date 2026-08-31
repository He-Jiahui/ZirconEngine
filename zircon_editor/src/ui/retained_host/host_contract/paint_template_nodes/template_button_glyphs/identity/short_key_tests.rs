use std::hint::black_box;
use std::time::Instant;

use super::{button_glyph_for_key, ButtonGlyph};

const CHECKS_PER_SAMPLE: usize = 1_000_000;
const SAMPLE_PAIRS: usize = 31;

#[inline(never)]
fn legacy_button_glyph_for_key(key: &str) -> ButtonGlyph {
    if key.contains("delete") || key.contains("trash") || key.contains("danger") {
        ButtonGlyph::Trash
    } else if key.contains("dropdown") || key.contains("drop-down") || key.contains("menu") {
        ButtonGlyph::ChevronDown
    } else if key.contains("icon") || key.contains("add") || key.contains("plus") {
        ButtonGlyph::Plus
    } else {
        ButtonGlyph::None
    }
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        let key = black_box("");
        let glyph = if optimized {
            button_glyph_for_key(key)
        } else {
            legacy_button_glyph_for_key(key)
        };
        evidence = evidence.wrapping_add(black_box(glyph) as usize);
    }
    black_box(evidence);
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

#[test]
fn optimization_batch_20260829ca_editor299_short_key_fast_path_preserves_results() {
    for key in [
        "",
        "a",
        "ab",
        "add",
        "menu",
        "danger-button",
        "plain-button",
    ] {
        assert_eq!(button_glyph_for_key(key), legacy_button_glyph_for_key(key));
    }
}

#[test]
fn optimization_batch_20260829ca_editor299_production_short_circuits_short_keys() {
    let source = include_str!("../identity.rs");
    let short_key_guard = source.find("if key.len() < 3").expect("short-key guard");
    let first_contains = source.find("key.contains").expect("glyph keyword scan");
    assert!(short_key_guard < first_contains);
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260829ca_editor299_short_key_benchmark() {
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(false));
            candidate.push(measure(true));
        } else {
            candidate.push(measure(true));
            baseline.push(measure(false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "EDITOR299_SHORT_GLYPH_KEY_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
