use std::hint::black_box;
use std::time::Instant;

use super::{badge_display_text, badge_root_label};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826fw_editor164_badge_display_preserves_fallback_and_trim() {
    let fallback = TemplatePaneNodeData {
        validation_message: "  validation fallback  ".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(badge_display_text(&fallback), "validation fallback");

    let preferred = TemplatePaneNodeData {
        value_text: "  value preferred  ".into(),
        validation_message: "validation fallback".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(badge_display_text(&preferred), "value preferred");
    let _: String = badge_root_label(&preferred);
}

#[test]
fn optimization_batch_20260826fw_editor164_only_overlay_display_becomes_borrowed() {
    let source = include_str!("../labels.rs");
    let display = source
        .find("fn badge_display_text(")
        .expect("display helper");
    let tests = source.find("#[cfg(test)]").expect("test module");
    let display_source = &source[display..tests];

    assert!(display_source.contains(") -> &str"));
    assert!(!display_source.contains(".to_string()"));
    assert!(source.contains("fn badge_root_label("));
    assert!(source.contains(") -> String"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fw_editor164_badge_display_borrow_bench() {
    let label = "b".repeat(LABEL_BYTES);
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
        "EDITOR164_BADGE_DISPLAY_BORROW_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} label_bytes={LABEL_BYTES} \
legacy_clones_per_drawn_badge=2 optimized_clones_per_drawn_badge=1 \
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
