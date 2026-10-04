use super::*;

#[test]
fn timing_status_keeps_disabled_unavailable_and_deferred_distinct() {
    assert_eq!(
        render_gpu_timing_status(false, false, None),
        RenderGpuTimingStatus::Disabled
    );
    assert_eq!(
        render_gpu_timing_status(true, false, None),
        RenderGpuTimingStatus::Unavailable
    );
    assert_eq!(
        render_gpu_timing_status(true, true, None),
        RenderGpuTimingStatus::Deferred
    );
}

#[test]
fn timing_status_projects_each_rhi_frame_observation() {
    let observation = |status| GpuTimerFrameObservation {
        frame_generation: 8,
        status,
    };

    assert_eq!(
        render_gpu_timing_status(true, true, Some(observation(GpuTimerFrameStatus::Pending))),
        RenderGpuTimingStatus::Pending
    );
    assert_eq!(
        render_gpu_timing_status(
            true,
            true,
            Some(observation(GpuTimerFrameStatus::CapacityExhausted))
        ),
        RenderGpuTimingStatus::CapacityExhausted
    );
    assert_eq!(
        render_gpu_timing_status(true, true, Some(observation(GpuTimerFrameStatus::NoPasses))),
        RenderGpuTimingStatus::NoPasses
    );
}
