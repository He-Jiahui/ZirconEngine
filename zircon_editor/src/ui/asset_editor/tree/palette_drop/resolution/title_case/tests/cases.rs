use std::hint::black_box;
use std::time::{Duration, Instant};

use super::title_case_identifier;

const SAMPLE_PAIRS: usize = 101;
const LABELS_PER_SAMPLE: usize = 4_096;

#[test]
fn editor897_palette_slot_title_preserves_text_for_all_delimiters() {
    for value in [
        "",
        "_",
        "_--_",
        "中文🦀",
        "\0slot__name",
        "UPPERcase",
        "hello_world",
        "  link-Root  ",
        "one TWO/three",
        "already Title Case",
        "第一项-héllo-🦀-LAST",
    ] {
        assert_eq!(
            title_case_identifier(value),
            legacy_title(value),
            "{value:?}"
        );
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor897_palette_slot_title_direct_release_percentiles() {
    let label = (0..16)
        .map(|index| format!("longword{index:02}"))
        .collect::<Vec<_>>()
        .join("--");
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&label, legacy_title));
            optimized.push(measure(&label, title_case_identifier));
        } else {
            optimized.push(measure(&label, title_case_identifier));
            legacy.push(measure(&label, legacy_title));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "EDITOR897_PALETTE_SLOT_TITLE_DIRECT_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn legacy_title(value: &str) -> String {
    let words = value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!(
                "{}{}",
                first.to_ascii_uppercase(),
                chars.as_str().to_ascii_lowercase()
            )
        })
        .collect::<Vec<_>>();
    if words.is_empty() {
        value.to_string()
    } else {
        words.join(" ")
    }
}

fn measure(label: &str, transform: fn(&str) -> String) -> Duration {
    let started = Instant::now();
    let checksum = (0..LABELS_PER_SAMPLE)
        .map(|_| black_box(transform(black_box(label))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
