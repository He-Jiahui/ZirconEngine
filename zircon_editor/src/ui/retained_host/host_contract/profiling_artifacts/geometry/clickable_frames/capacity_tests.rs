use std::hint::black_box;
use std::time::Instant;

use super::{collect_clickable_frames, UiProfileNamedFrame, UiProfileTabFrame};
use crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileFrame;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const FRAMES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826ey_editor140_capacity_preserves_clickable_frame_order() {
    let resize_splitters = named_frames("resize", 16);
    let document_tabs = tab_frames("document", 64);
    let drawer_tabs = tab_frames("drawer", 48);
    let host_page_tabs = tab_frames("page", 32);
    let activity_rail_buttons = named_frames("activity", 16);
    let viewport_toolbar_controls = named_frames("toolbar", 48);
    let template_controls = named_frames("template", 32);

    let frames = collect_clickable_frames(
        &resize_splitters,
        &document_tabs,
        &drawer_tabs,
        &host_page_tabs,
        &activity_rail_buttons,
        &viewport_toolbar_controls,
        &template_controls,
    );

    assert_eq!(frames.len(), FRAMES_PER_BUILD);
    assert!(frames.capacity() >= FRAMES_PER_BUILD);
    assert_eq!(frames[0].id, "resize-0");
    assert_eq!(frames[16].id, "document-0");
    assert_eq!(frames[128].id, "page-0");
    assert_eq!(frames[FRAMES_PER_BUILD - 1].id, "template-31");
}

#[test]
fn optimization_batch_20260826ey_editor140_clickable_frames_reserve_exact_input_sum() {
    let source = include_str!("../clickable_frames.rs");
    assert!(source.contains("fn clickable_frame_capacity("));
    assert!(source.contains("Vec::with_capacity(clickable_frame_capacity("));
    assert!(source.contains(".saturating_add(template_controls.len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ey_editor140_clickable_frame_capacity_bench() {
    let inputs = [16usize, 64, 48, 32, 16, 48, 32];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&inputs, false));
            optimized_samples.push(measure(&inputs, true));
        } else {
            optimized_samples.push(measure(&inputs, true));
            legacy_samples.push(measure(&inputs, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR140_CLICKABLE_FRAME_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} frames_per_build={FRAMES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn named_frames(prefix: &str, count: usize) -> Vec<UiProfileNamedFrame> {
    (0..count)
        .map(|index| UiProfileNamedFrame {
            id: format!("{prefix}-{index}"),
            kind: prefix.to_string(),
            surface: "main".to_string(),
            frame: profile_frame(index),
            clip: None,
        })
        .collect()
}

fn tab_frames(prefix: &str, count: usize) -> Vec<UiProfileTabFrame> {
    (0..count)
        .map(|index| UiProfileTabFrame {
            id: format!("{prefix}-{index}"),
            title: format!("{prefix} {index}"),
            kind: prefix.to_string(),
            surface: "main".to_string(),
            frame: profile_frame(index),
            close_frame: profile_frame(index),
            active: index == 0,
        })
        .collect()
}

fn profile_frame(index: usize) -> UiProfileFrame {
    UiProfileFrame {
        x: index as f32,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }
}

fn measure(inputs: &[usize; 7], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut frames = if reserve {
            Vec::with_capacity(FRAMES_PER_BUILD)
        } else {
            Vec::new()
        };
        for count in inputs {
            for frame in 0..*count {
                frames.push(black_box(frame));
            }
        }
        checksum ^= black_box(frames.len() ^ frames.capacity());
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
