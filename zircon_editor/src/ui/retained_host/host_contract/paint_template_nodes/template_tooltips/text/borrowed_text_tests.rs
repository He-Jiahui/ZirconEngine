use std::hint::black_box;
use std::time::Instant;

use super::{tooltip_body, tooltip_title};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const SAMPLE_PAIRS: usize = 21;
const CALLS_PER_SAMPLE: usize = 131_072;

fn fixture() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        text: "  Build Project  ".into(),
        label_text: "  Ctrl+Shift+B  ".into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn optimization_batch_20260826dc_editor92_tooltip_text_preserves_trim_and_fallback() {
    let node = fixture();
    assert_eq!(tooltip_title(&node), "Build Project");
    assert_eq!(tooltip_body(&node), "Ctrl+Shift+B");

    let empty = TemplatePaneNodeData {
        text: "  \t ".into(),
        label_text: " \n ".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(tooltip_title(&empty), "Tooltip");
    assert_eq!(tooltip_body(&empty), "");
}

#[test]
fn optimization_batch_20260826dc_editor92_tooltip_text_borrows_node_storage() {
    let node = fixture();
    let expected_title = node.text.as_str().trim();
    let expected_body = node.label_text.as_str().trim();
    let title = tooltip_title(&node);
    let body = tooltip_body(&node);

    assert_eq!(title.as_ptr(), expected_title.as_ptr());
    assert_eq!(body.as_ptr(), expected_body.as_ptr());

    let text_source = include_str!("../text.rs");
    let layout_source = include_str!("../layout.rs");
    let title_source = include_str!("title.rs");
    let body_source = include_str!("body.rs");
    assert!(!text_source.contains("trim().to_string()"));
    assert!(layout_source.contains("measure_runtime_text_width(tooltip_title(node)"));
    assert!(layout_source.contains("measure_runtime_text_width(tooltip_body(node)"));
    assert!(title_source.contains("title.to_string()"));
    assert!(body_source.contains("body.to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dc_editor92_tooltip_borrowed_text_bench() {
    let node = fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&node));
            optimized_samples.push(measure_optimized(&node));
        } else {
            optimized_samples.push(measure_optimized(&node));
            legacy_samples.push(measure_legacy(&node));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR92_TOOLTIP_BORROWED_TEXT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} text_resolutions_per_call=2 \
legacy_layout_allocations_per_sample={} optimized_layout_allocations_per_sample=0 \
paint_command_owned_allocations_per_call=2 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        CALLS_PER_SAMPLE * 2,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed tooltip text P95 {optimized_p95_ns}ns must be at most 70% of owned layout text P95 {legacy_p95_ns}ns"
    );
}

fn legacy_tooltip_title(node: &TemplatePaneNodeData) -> String {
    let text = node.text.as_str().trim();
    if text.is_empty() {
        "Tooltip".to_string()
    } else {
        text.to_string()
    }
}

fn legacy_tooltip_body(node: &TemplatePaneNodeData) -> String {
    node.label_text.as_str().trim().to_string()
}

fn measure_legacy(node: &TemplatePaneNodeData) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CALLS_PER_SAMPLE {
        checksum ^= black_box(legacy_tooltip_title(black_box(node))).len();
        checksum ^= black_box(legacy_tooltip_body(black_box(node))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(node: &TemplatePaneNodeData) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CALLS_PER_SAMPLE {
        checksum ^= black_box(tooltip_title(black_box(node))).len();
        checksum ^= black_box(tooltip_body(black_box(node))).len();
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
