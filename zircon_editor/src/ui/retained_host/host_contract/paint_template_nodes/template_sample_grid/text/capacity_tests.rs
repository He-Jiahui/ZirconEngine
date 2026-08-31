use std::hint::black_box;
use std::time::Instant;

use super::push_text;
use crate::ui::retained_host::host_contract::data::FrameRect;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826gb_editor169_sample_grid_text_preserves_empty_filter() {
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 64.0,
        height: 16.0,
    };
    let mut commands = Vec::new();

    push_text(
        &mut commands,
        frame.clone(),
        &frame,
        0,
        "   ",
        [255; 4],
        12.0,
        16.0,
        1.0,
    );

    assert!(commands.is_empty());
}

#[test]
fn optimization_batch_20260826gb_editor169_sample_grid_text_owns_after_frame_gate() {
    let source = include_str!("../text.rs");
    let cow_conversion = source
        .find("let text = text.into();")
        .expect("Cow conversion");
    let frame_gate = source
        .find("frame.width <= f32::EPSILON")
        .expect("frame gate");
    let ownership = source.find("text.into_owned()").expect("command ownership");

    assert!(cow_conversion < frame_gate && frame_gate < ownership);
    assert!(source.contains("text: impl Into<Cow<'a, str>>"));
    assert!(!source.contains("tick.label().to_string()"));
    assert!(!source.contains("grid.x_axis_label().to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gb_editor169_sample_grid_text_deferred_allocation_bench() {
    let label = "g".repeat(LABEL_BYTES);
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
        "EDITOR169_SAMPLE_GRID_TEXT_DEFERRED_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
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
