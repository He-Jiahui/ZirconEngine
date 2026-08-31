use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

use super::{collect_tabs, FrameRect, HostChromeTabData, ModelRc};
use crate::ui::retained_host::primitives::VecModel;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const TABS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fg_editor148_capacity_preserves_profile_tab_frames() {
    let rows = (0..TABS_PER_BUILD)
        .map(|index| HostChromeTabData {
            control_id: format!("tab-{index:03}").into(),
            frame: FrameRect {
                x: index as f32,
                y: 2.0,
                width: 80.0,
                height: 24.0,
            },
            close_frame: FrameRect {
                x: index as f32 + 60.0,
                y: 4.0,
                width: 16.0,
                height: 16.0,
            },
            ..HostChromeTabData::default()
        })
        .collect::<Vec<_>>();
    let tabs = ModelRc::from(Rc::new(VecModel::from(rows)));
    let origin = FrameRect {
        x: 10.0,
        y: 20.0,
        ..FrameRect::default()
    };

    let frames = collect_tabs("document_tab", "document", &tabs, &origin);

    assert_eq!(frames.len(), TABS_PER_BUILD);
    assert!(frames.capacity() >= TABS_PER_BUILD);
    assert_eq!(frames[0].id, "tab-000");
    assert_eq!(frames[TABS_PER_BUILD - 1].id, "tab-255");
    assert_eq!(frames[0].frame.x, 10.0);
    assert_eq!(frames[0].frame.y, 22.0);
    assert_eq!(frames[0].close_frame.x, 70.0);
}

#[test]
fn optimization_batch_20260826fg_editor148_profile_tabs_reserve_model_rows() {
    let source = include_str!("../collect.rs");
    assert!(source.contains("Vec::with_capacity(tabs.row_count())"));
    assert!(source.contains("for row in 0..tabs.row_count()"));
    assert!(source.contains("if !is_visible_frame(&frame)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fg_editor148_profile_tab_frame_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR148_PROFILE_TAB_FRAME_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} tabs_per_build={TABS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut frames = if reserve {
            Vec::with_capacity(TABS_PER_BUILD)
        } else {
            Vec::new()
        };
        for frame in 0..TABS_PER_BUILD {
            frames.push(black_box(frame));
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
