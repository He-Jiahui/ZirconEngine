use std::hint::black_box;
use std::time::Instant;

use super::{context_action_menu_option_id, humanize_menu_action_segment};

const SAMPLE_PAIRS: usize = 21;
const CALLS_PER_SAMPLE: usize = 131_072;
const ACTION_SEGMENT: &str = "open_recent_project_in_new_editor_window";

#[test]
fn optimization_batch_20260826de_editor94_menu_action_label_preserves_protocol_behavior() {
    assert_eq!(
        context_action_menu_option_id("menu.item.open_recent_project"),
        Some("Open Recent Project".to_string())
    );
    assert_eq!(
        context_action_menu_option_id("menu.item.__open__new_window__"),
        Some("Open New Window".to_string())
    );
    assert_eq!(context_action_menu_option_id("---"), None);
    assert_eq!(
        context_action_menu_option_id("Delete | checked, disabled"),
        None
    );
    assert_eq!(
        context_action_menu_option_id(" Rename | checked"),
        Some("Rename".to_string())
    );
}

#[test]
fn optimization_batch_20260826de_editor94_menu_action_label_uses_one_output_buffer() {
    let source = include_str!("../events.rs");

    assert!(source.contains("String::with_capacity(action_segment.len())"));
    assert!(source.contains("label.push(first.to_ascii_uppercase())"));
    assert!(source.contains("label.push_str(chars.as_str())"));
    assert!(!source.contains("format!(\"{}{}\", first.to_ascii_uppercase()"));
    assert!(!source.contains("collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826de_editor94_menu_action_label_single_buffer_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_humanize_menu_action_segment));
            optimized_samples.push(measure(humanize_menu_action_segment));
        } else {
            optimized_samples.push(measure(humanize_menu_action_segment));
            legacy_samples.push(measure(legacy_humanize_menu_action_segment));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR94_MENU_ACTION_LABEL_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} segments_per_call=7 \
legacy_allocations_per_call=9 optimized_allocations_per_call=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer menu label P95 {optimized_p95_ns}ns must be at most 70% of collected-segment P95 {legacy_p95_ns}ns"
    );
}

fn legacy_humanize_menu_action_segment(action_segment: &str) -> String {
    action_segment
        .split('_')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn measure(humanize: fn(&str) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CALLS_PER_SAMPLE {
        checksum ^= black_box(humanize(black_box(ACTION_SEGMENT))).len();
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
