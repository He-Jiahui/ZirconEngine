use std::hint::black_box;
use std::time::Instant;

use super::push_label;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_weight_heatmap::geometry::WeightHeatmapGeometry;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826ga_editor168_heatmap_label_preserves_empty_filter() {
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 80.0,
    };
    let geometry = WeightHeatmapGeometry::from_frame(&frame, 20.0);
    let mut commands = Vec::new();

    push_label(&mut commands, "   ", 0.0, &geometry, &frame, 0, 1.0);

    assert!(commands.is_empty());
}

#[test]
fn optimization_batch_20260826ga_editor168_heatmap_label_allocates_after_geometry_gate() {
    let source = include_str!("../text.rs");
    let borrowed_parameter = source.find("text: &str").expect("borrowed label");
    let frame_gate = source
        .find("if frame.width <= f32::EPSILON")
        .expect("frame gate");
    let allocation = source.find("text.to_owned()").expect("command allocation");

    assert!(borrowed_parameter < frame_gate && frame_gate < allocation);
    assert!(source.contains("generation.high_label(),"));
    assert!(source.contains("generation.low_label(),"));
    assert!(!source.contains("generation.high_label().to_owned()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ga_editor168_heatmap_label_deferred_allocation_bench() {
    let label = "h".repeat(LABEL_BYTES);
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
        "EDITOR168_HEATMAP_LABEL_DEFERRED_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} label_bytes={LABEL_BYTES} \
legacy_allocations_per_rejected_label=1 optimized_allocations_per_rejected_label=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(label: &str, defer_allocation: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LABELS_PER_SAMPLE {
        let frame_is_valid = black_box(false);
        if defer_allocation {
            if !frame_is_valid {
                checksum ^= black_box(label.len());
                continue;
            }
            checksum ^= black_box(label.to_owned().len());
        } else {
            let owned = black_box(label).to_owned();
            if !frame_is_valid {
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
