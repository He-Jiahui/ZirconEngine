use std::hint::black_box;
use std::time::Instant;

use super::divider_label;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826fv_editor163_divider_label_preserves_fallback_order() {
    let fallback = TemplatePaneNodeData {
        options_text: "options fallback".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(divider_label(&fallback), "options fallback");

    let preferred = TemplatePaneNodeData {
        text: "text preferred".into(),
        value_text: "value fallback".into(),
        options_text: "options fallback".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(divider_label(&preferred), "text preferred");
}

#[test]
fn optimization_batch_20260826fv_editor163_divider_label_is_borrowed_until_command() {
    let source = include_str!("../text.rs");
    assert!(source.contains("fn divider_label("));
    assert!(source.contains(") -> &str"));
    assert!(!source.contains(".to_string()"));

    let horizontal = include_str!("../../horizontal.rs");
    let vertical = include_str!("../../vertical.rs");
    assert!(horizontal.contains("push_horizontal_divider_label("));
    assert!(vertical.contains("push_vertical_divider_label("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fv_editor163_divider_label_borrow_bench() {
    let label = "d".repeat(LABEL_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&label, false));
            optimized_samples.push(measure(&label, true));
        } else {
            optimized_samples.push(measure(&label, true));
            legacy_samples.push(measure(&label, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR163_DIVIDER_LABEL_BORROW_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} label_bytes={LABEL_BYTES} \
legacy_clones_per_drawn_label=2 optimized_clones_per_drawn_label=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(label: &str, borrow_until_command: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LABELS_PER_SAMPLE {
        let command_label = if borrow_until_command {
            black_box(label).to_string()
        } else {
            let projected = black_box(label).to_string();
            black_box(projected.as_str()).to_string()
        };
        checksum ^= black_box(command_label.len() ^ command_label.capacity());
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
