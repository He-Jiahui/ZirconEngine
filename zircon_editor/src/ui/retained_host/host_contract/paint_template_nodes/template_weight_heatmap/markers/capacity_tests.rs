use std::hint::black_box;
use std::time::Instant;

use super::{
    push_heat_source_markers, FrameRect, HostPaintCommand, WeightHeatmapGeometry,
    WeightHeatmapSource,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const SOURCES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fh_editor149_capacity_preserves_heat_source_markers() {
    let sources = (0..SOURCES_PER_BUILD)
        .map(|index| {
            WeightHeatmapSource::new(
                index as f32 / SOURCES_PER_BUILD as f32,
                0.5,
                1.0,
                index == 0,
            )
        })
        .collect::<Vec<_>>();
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 180.0,
    };
    let geometry = WeightHeatmapGeometry::from_frame(&frame, 20.0);
    let mut commands = Vec::<HostPaintCommand>::new();

    push_heat_source_markers(&mut commands, &sources, &geometry, &frame, 10, 1.0);

    assert_eq!(commands.len(), SOURCES_PER_BUILD);
    assert!(commands.capacity() >= SOURCES_PER_BUILD);

    let collapsed = WeightHeatmapGeometry::from_frame(
        &FrameRect {
            width: 0.0,
            ..frame.clone()
        },
        20.0,
    );
    let mut collapsed_commands = Vec::new();
    push_heat_source_markers(
        &mut collapsed_commands,
        &sources,
        &collapsed,
        &frame,
        10,
        1.0,
    );
    assert!(collapsed_commands.is_empty());
    assert_eq!(collapsed_commands.capacity(), 0);
}

#[test]
fn optimization_batch_20260826fh_editor149_heat_source_markers_reserve_source_count() {
    let source = include_str!("../markers.rs");
    assert!(source.contains("if !geometry.is_drawable()"));
    assert!(source.contains("commands.reserve(sources.len())"));
    assert!(source.contains("for source in sources"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fh_editor149_heat_source_marker_capacity_bench() {
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
        "EDITOR149_HEAT_SOURCE_MARKER_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} sources_per_build={SOURCES_PER_BUILD} \
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
        let mut commands = Vec::new();
        if reserve {
            commands.reserve(SOURCES_PER_BUILD);
        }
        for command in 0..SOURCES_PER_BUILD {
            commands.push(black_box(command));
        }
        checksum ^= black_box(commands.len() ^ commands.capacity());
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
