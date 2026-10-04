use std::time::Duration;

use super::{copy_rgba_to_xrgb, duration_micros, ReferenceCpuPresenterMetrics};

#[test]
fn complete_rgba_frame_overwrites_the_surface_without_a_preclear() {
    let mut surface = [0x00ff_00ff, 0x00ff_00ff];

    let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255, 4, 5, 6, 255]);

    assert!(!cleared);
    assert_eq!(surface, [0x0001_0203, 0x0004_0506]);
}

#[test]
fn truncated_rgba_frame_clears_uncovered_surface_pixels() {
    let mut surface = [0x00ff_00ff, 0x00ff_00ff];

    let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255]);

    assert!(cleared);
    assert_eq!(surface, [0x0001_0203, 0]);
}

#[test]
fn reference_cpu_metrics_accumulate_copy_latency_and_drops_without_overflowing() {
    let mut metrics = ReferenceCpuPresenterMetrics::default();

    metrics.record_copied_bytes(1024);
    metrics.record_presented(Duration::from_micros(73));
    metrics.record_dropped_frame();

    assert_eq!(metrics.presented_frames, 1);
    assert_eq!(metrics.copied_bytes, 1024);
    assert_eq!(metrics.total_latency_micros, 73);
    assert_eq!(metrics.last_latency_micros, 73);
    assert_eq!(metrics.dropped_frames, 1);
}

#[test]
fn reference_cpu_latency_conversion_saturates_at_u64_max() {
    assert_eq!(
        duration_micros(Duration::MAX),
        u64::MAX,
        "long-running diagnostic capture should not wrap its latency counter"
    );
}
