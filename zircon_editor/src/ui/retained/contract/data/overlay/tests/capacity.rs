use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::math::{UVec2, Vec2, Vec4};

use crate::scene::viewport::HandleScreenLine;

use super::{clipped_raster_lines, RasterLine};

const LINE_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;
const SOURCE_SIZE: UVec2 = UVec2::new(512, 512);

#[test]
fn editor881_clipped_line_capacity_keeps_all_rejected_path_lazy() {
    let rejected = (0..64)
        .map(|index| {
            let coordinate = -1_000.0 - index as f32;
            screen_line(
                Vec2::new(coordinate, coordinate),
                Vec2::new(coordinate - 1.0, coordinate - 1.0),
            )
        })
        .collect::<Vec<_>>();

    let clipped = clipped_raster_lines(&rejected, SOURCE_SIZE);

    assert!(clipped.is_empty());
    assert_eq!(clipped.capacity(), 0);
}

#[test]
fn editor881_clipped_line_capacity_preserves_retained_order() {
    let mut lines = Vec::with_capacity(72);
    for index in 0..8 {
        let coordinate = -1_000.0 - index as f32;
        lines.push(screen_line(
            Vec2::new(coordinate, coordinate),
            Vec2::new(coordinate - 1.0, coordinate - 1.0),
        ));
    }
    for index in 0..64 {
        let coordinate = index as f32;
        lines.push(screen_line(
            Vec2::new(coordinate, 32.0),
            Vec2::new(coordinate + 1.0, 33.0),
        ));
    }

    let clipped = clipped_raster_lines(&lines, SOURCE_SIZE);

    assert_eq!(clipped.len(), 64);
    assert!(clipped.capacity() >= lines.len());
    assert_eq!(clipped[0].start, Vec2::new(0.0, 32.0));
    assert_eq!(clipped[63].start, Vec2::new(63.0, 32.0));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor881_viewport_overlay_clipped_line_capacity_benchmark() {
    let lines = dense_lines();
    let legacy_output = legacy_clipped_raster_lines(&lines, SOURCE_SIZE);
    let optimized_output = clipped_raster_lines(&lines, SOURCE_SIZE);
    assert_eq!(
        raster_checksum(&legacy_output),
        raster_checksum(&optimized_output)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_clipped_raster_lines(&lines, SOURCE_SIZE)));
            optimized.push(measure(|| clipped_raster_lines(&lines, SOURCE_SIZE)));
        } else {
            optimized.push(measure(|| clipped_raster_lines(&lines, SOURCE_SIZE)));
            legacy.push(measure(|| legacy_clipped_raster_lines(&lines, SOURCE_SIZE)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR881_VIEWPORT_OVERLAY_CLIPPED_LINE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} lines={LINE_COUNT} legacy_growth_events=11 optimized_growth_events=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(growth_events(LINE_COUNT, 0), 11);
    assert_eq!(growth_events(LINE_COUNT, LINE_COUNT), 0);
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "lazy-reserved clipping P95 {optimized_p95}ns must stay within 10% of filtered collection P95 {legacy_p95}ns"
    );
}

fn dense_lines() -> Vec<HandleScreenLine> {
    (0..LINE_COUNT)
        .map(|index| {
            let x = (index % 256) as f32;
            let y = (index / 256) as f32;
            screen_line(Vec2::new(x, y), Vec2::new(x + 1.0, y + 1.0))
        })
        .collect()
}

fn screen_line(start: Vec2, end: Vec2) -> HandleScreenLine {
    HandleScreenLine::new(start, end, Vec4::ONE, 2.0, None)
}

fn legacy_clipped_raster_lines(lines: &[HandleScreenLine], source_size: UVec2) -> Vec<RasterLine> {
    lines
        .iter()
        .copied()
        .filter_map(|line| RasterLine::clipped(line, source_size))
        .collect()
}

fn raster_checksum(lines: &[RasterLine]) -> u64 {
    lines.iter().fold(0u64, |checksum, line| {
        checksum
            .wrapping_add(u64::from(line.start.x.to_bits()))
            .wrapping_add(u64::from(line.start.y.to_bits()))
            .wrapping_add(u64::from(line.end.x.to_bits()))
            .wrapping_add(u64::from(line.end.y.to_bits()))
            .wrapping_add(u64::from(line.width.to_bits()))
            .wrapping_add(
                line.color
                    .iter()
                    .map(|channel| u64::from(*channel))
                    .sum::<u64>(),
            )
    })
}

fn growth_events(item_count: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
