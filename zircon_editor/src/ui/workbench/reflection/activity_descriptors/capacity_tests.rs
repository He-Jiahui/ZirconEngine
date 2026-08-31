use std::hint::black_box;
use std::time::Instant;

use super::{
    activity_descriptor_capacities, activity_descriptors_from_views, ActivityViewDescriptor,
    ActivityWindowDescriptor, ViewDescriptor, ViewDescriptorId, ViewKind,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const VIEWS_PER_BUILD: usize = 128;
const WINDOWS_PER_BUILD: usize = 128;

#[test]
fn optimization_batch_20260826ev_editor137_capacity_preserves_activity_projection() {
    let descriptors = (0..VIEWS_PER_BUILD)
        .map(|index| descriptor(index, ViewKind::ActivityView))
        .chain(
            (0..WINDOWS_PER_BUILD)
                .map(|index| descriptor(VIEWS_PER_BUILD + index, ViewKind::ActivityWindow)),
        )
        .collect::<Vec<_>>();

    let (views, windows) = activity_descriptors_from_views(&descriptors);

    assert_eq!(views.len(), VIEWS_PER_BUILD);
    assert!(views.capacity() >= VIEWS_PER_BUILD);
    assert_eq!(windows.len(), WINDOWS_PER_BUILD);
    assert!(windows.capacity() >= WINDOWS_PER_BUILD);
    assert_eq!(views[0].view_id, "editor.capacity-0");
    assert_eq!(views[127].view_id, "editor.capacity-127");
    assert_eq!(windows[0].window_id, "editor.capacity-128");
    assert_eq!(windows[127].window_id, "editor.capacity-255");
    assert_eq!(
        activity_descriptor_capacities(&descriptors),
        (VIEWS_PER_BUILD, WINDOWS_PER_BUILD)
    );
}

#[test]
fn optimization_batch_20260826ev_editor137_activity_projection_reserves_kind_counts() {
    let source = include_str!("../activity_descriptors.rs");
    assert!(source.contains("activity_descriptor_capacities(descriptors)"));
    assert!(source.contains("Vec::with_capacity(view_count)"));
    assert!(source.contains("Vec::with_capacity(window_count)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ev_editor137_activity_descriptor_capacity_bench() {
    let view = ActivityViewDescriptor::new("view", "View", "view");
    let window = ActivityWindowDescriptor::new("window", "Window", "window");
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&view, &window, false));
            optimized_samples.push(measure(&view, &window, true));
        } else {
            optimized_samples.push(measure(&view, &window, true));
            legacy_samples.push(measure(&view, &window, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR137_ACTIVITY_DESCRIPTOR_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} views_per_build={VIEWS_PER_BUILD} \
windows_per_build={WINDOWS_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=2 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn descriptor(index: usize, kind: ViewKind) -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(format!("editor.capacity-{index}")),
        kind,
        format!("Capacity {index}"),
    )
}

fn measure(
    view: &ActivityViewDescriptor,
    window: &ActivityWindowDescriptor,
    reserve: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut views = if reserve {
            Vec::with_capacity(VIEWS_PER_BUILD)
        } else {
            Vec::new()
        };
        let mut windows = if reserve {
            Vec::with_capacity(WINDOWS_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..VIEWS_PER_BUILD {
            views.push(black_box(view.clone()));
        }
        for _ in 0..WINDOWS_PER_BUILD {
            windows.push(black_box(window.clone()));
        }
        checksum ^= black_box(views.len() ^ views.capacity() ^ windows.len() ^ windows.capacity());
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
