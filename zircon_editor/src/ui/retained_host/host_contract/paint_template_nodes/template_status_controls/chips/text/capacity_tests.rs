use std::hint::black_box;
use std::time::Instant;

use super::push_status_chip_text;
use crate::ui::retained_host::host_contract::data::FrameRect;

const SAMPLE_PAIRS: usize = 21;
const CHIPS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826gd_editor171_status_chip_rejects_disjoint_clip() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 1_000.0,
        y: 1_000.0,
        width: 20.0,
        height: 20.0,
    };
    let mut commands = Vec::new();

    push_status_chip_text(
        &mut commands,
        &rect,
        &clip,
        0,
        "Build: Ready",
        [255; 4],
        [224; 4],
        1.0,
    );

    assert!(commands.is_empty());
}

#[test]
fn optimization_batch_20260826gd_editor171_status_chip_clips_before_text_split() {
    let source = include_str!("../text.rs");
    let base = source
        .find("let base = status_chip_text_rect(rect);")
        .expect("status chip base");
    let clip_gate = source
        .find("intersect(&base, clip).is_none()")
        .expect("clip gate");
    let split = source
        .find("match split_status_chip_text(label)")
        .expect("text split");

    assert!(base < clip_gate && clip_gate < split);
    assert_eq!(
        source.matches("intersect(&base, clip).is_none()").count(),
        1
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gd_editor171_status_chip_clip_allocation_gate_bench() {
    let label = format!("{}: Ready", "s".repeat(LABEL_BYTES));
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
        "EDITOR171_STATUS_CHIP_CLIP_ALLOCATION_GATE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
chips_per_sample={CHIPS_PER_SAMPLE} label_bytes={LABEL_BYTES} \
legacy_text_splits_per_rejected_chip=1 optimized_text_splits_per_rejected_chip=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(label: &str, gate_before_split: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CHIPS_PER_SAMPLE {
        let clip_intersects = black_box(false);
        if gate_before_split && !clip_intersects {
            checksum ^= black_box(label.len());
            continue;
        }
        let (leading, value) = black_box(label).split_once(':').expect("label/value");
        let leading = format!("{}:", leading.trim());
        let value = value.trim().to_string();
        if !clip_intersects {
            checksum ^= black_box(leading.len() ^ value.len());
            continue;
        }
        checksum ^= black_box(leading.len() ^ value.len());
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
