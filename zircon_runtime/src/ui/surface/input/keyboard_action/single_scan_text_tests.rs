use std::hint::black_box;
use std::time::Instant;

use super::keyboard_text_is_usable;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const TEXT_CHARS: usize = 4096;

fn legacy_keyboard_text_is_usable(text: &str) -> bool {
    !(text.is_empty()
        || text.chars().any(char::is_control)
        || text.chars().all(char::is_whitespace))
}

fn measure(text: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut usable = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        usable += usize::from(if optimized {
            keyboard_text_is_usable(black_box(text))
        } else {
            legacy_keyboard_text_is_usable(black_box(text))
        });
    }
    black_box(usable);
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
fn runtime_hotpath_batch_runtime337_338_keyboard_text_preserves_results() {
    for text in [
        "", " ", "   ", "\u{2003}", "\n", "value", " value ", "\u{4f8b}", "value\n",
    ] {
        assert_eq!(
            keyboard_text_is_usable(text),
            legacy_keyboard_text_is_usable(text),
            "{text:?}"
        );
    }
}

#[test]
fn runtime_hotpath_batch_runtime337_338_keyboard_text_uses_one_char_scan() {
    let source = include_str!("../keyboard_action.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("for character in text.chars()"));
    assert!(!production.contains("text.chars().any(char::is_control)"));
    assert!(!production.contains("text.chars().all(char::is_whitespace)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime337_338_single_scan_keyboard_text_bench() {
    let text = " ".repeat(TEXT_CHARS);
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&text, false));
            candidate_samples.push(measure(&text, true));
        } else {
            candidate_samples.push(measure(&text, true));
            baseline_samples.push(measure(&text, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "RUNTIME337_SINGLE_SCAN_KEYBOARD_TEXT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} text_chars={TEXT_CHARS} \
baseline_character_scans=2 candidate_character_scans=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
