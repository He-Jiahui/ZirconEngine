use super::{
    decode_timestamp_pairs, gpu_timestamp_features_supported, insert_completed_frame_in_order,
    take_oldest_completed_frame, timer_frame_status, timestamp_delta_us, GpuTimerFrameResult,
    GpuTimerFrameStatus, GPU_TIMESTAMP_REQUIRED_FEATURES,
};
use std::collections::VecDeque;

#[test]
fn render_perf_gpu_timer_capability_gate() {
    assert!(gpu_timestamp_features_supported(
        GPU_TIMESTAMP_REQUIRED_FEATURES | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
    ));
    assert!(!gpu_timestamp_features_supported(
        wgpu::Features::TIMESTAMP_QUERY
    ));
    assert!(!gpu_timestamp_features_supported(
        wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
    ));
}

#[test]
fn timestamp_pairs_decode_only_the_resolved_query_range() {
    let mut bytes = Vec::new();
    for timestamp in [10_u64, 20, 30, 50, 99, 100] {
        bytes.extend_from_slice(&timestamp.to_le_bytes());
    }

    assert_eq!(
        decode_timestamp_pairs(&bytes, 4),
        Some(vec![[10, 20], [30, 50]])
    );
}

#[test]
fn timestamp_delta_converts_queue_period_to_rounded_microseconds() {
    assert_eq!(timestamp_delta_us(100, 132, 2.5), 0);
    assert_eq!(timestamp_delta_us(100, 900, 2.5), 2);
    assert_eq!(timestamp_delta_us(900, 100, 2.5), 0);
}

#[test]
fn completed_timer_frames_are_drained_oldest_first_without_dropping_ready_results() {
    let mut completed_frames = VecDeque::new();
    for frame_generation in [4, 2, 3] {
        insert_completed_frame_in_order(
            &mut completed_frames,
            GpuTimerFrameResult {
                frame_generation,
                pass_timings: Vec::new(),
            },
        );
    }

    let drained_generations = std::iter::from_fn(|| {
        take_oldest_completed_frame(&mut completed_frames).map(|frame| frame.frame_generation)
    })
    .collect::<Vec<_>>();

    assert_eq!(drained_generations, vec![2, 3, 4]);
    assert!(take_oldest_completed_frame(&mut completed_frames).is_none());
}

#[test]
fn timer_observation_distinguishes_deferred_and_capacity_limited_frames() {
    assert_eq!(
        timer_frame_status(0, false, true),
        GpuTimerFrameStatus::NoPasses
    );
    assert_eq!(
        timer_frame_status(2, false, false),
        GpuTimerFrameStatus::Deferred
    );
    assert_eq!(
        timer_frame_status(2, true, true),
        GpuTimerFrameStatus::CapacityExhausted
    );
    assert_eq!(
        timer_frame_status(2, false, true),
        GpuTimerFrameStatus::Pending
    );
}

#[test]
fn timer_collector_only_drains_results_after_the_readback_owner_polls() {
    let source = include_str!("../gpu_pass_timer.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(source.contains("pub fn try_collect(&mut self)"));
    assert!(!source.contains("readback_queue.poll_completed"));
}

#[test]
fn product_timer_constructor_does_not_receive_queue_authority() {
    let source = include_str!("../gpu_pass_timer.rs");
    let product_constructor = source
        .split("pub fn try_new_product(")
        .nth(1)
        .and_then(|source| source.split("pub fn begin_frame").next())
        .expect("product timer constructor");

    assert!(!product_constructor.contains("wgpu::Queue"));
    assert!(!product_constructor.contains("get_timestamp_period"));
}
