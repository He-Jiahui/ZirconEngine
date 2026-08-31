use std::hint::black_box;
use std::time::Instant;

use super::chip_label;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826fu_editor162_chip_label_preserves_text_fallback() {
    let fallback = TemplatePaneNodeData {
        value_text: "value fallback".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(chip_label(&fallback), "value fallback");

    let preferred = TemplatePaneNodeData {
        text: "text preferred".into(),
        value_text: "value fallback".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(chip_label(&preferred), "text preferred");
}

#[test]
fn optimization_batch_20260826fu_editor162_chip_clone_occurs_after_frame_gate() {
    let source = include_str!("../text.rs");
    let borrow = source
        .find("let label = chip_label(node);")
        .expect("borrowed label");
    let frame = source
        .find("let Some((frame, font_size, line_height))")
        .expect("frame gate");
    let clone = source
        .find("label.to_string(),")
        .expect("command-owned label");

    assert!(source.contains("fn chip_label(node: &TemplatePaneNodeData) -> &str"));
    assert!(borrow < frame && frame < clone);
    assert_eq!(source.matches("label.to_string()").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fu_editor162_chip_label_deferred_clone_bench() {
    let label = "c".repeat(LABEL_BYTES);
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
        "EDITOR162_CHIP_LABEL_DEFERRED_CLONE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} label_bytes={LABEL_BYTES} \
legacy_clones_per_rejected_label=1 optimized_clones_per_rejected_label=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(label: &str, defer_clone: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LABELS_PER_SAMPLE {
        let frame_available = black_box(false);
        if defer_clone {
            let borrowed = black_box(label);
            if !frame_available {
                checksum ^= black_box(borrowed.len());
                continue;
            }
            checksum ^= black_box(borrowed.to_string().len());
        } else {
            let owned = black_box(label).to_string();
            if !frame_available {
                checksum ^= black_box(owned.len());
                continue;
            }
            checksum ^= black_box(owned.len());
        }
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
