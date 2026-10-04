use super::{
    GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
    GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
    GpuPipelineStatisticsTimer, GpuTimerFrameObservation, GpuTimerFrameResult, GpuTimerFrameStatus,
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
};

#[test]
fn gpu_timer_contract_is_available_from_the_backend_root() {
    let projected_type_sizes = [
        std::mem::size_of::<GpuPassTimer>(),
        std::mem::size_of::<GpuPassTimestampScope>(),
        std::mem::size_of::<GpuPassTiming>(),
        std::mem::size_of::<GpuTimerFrameObservation>(),
        std::mem::size_of::<GpuTimerFrameResult>(),
        std::mem::size_of::<GpuPipelineStatisticsTimer>(),
        std::mem::size_of::<GpuPipelineStatisticsScope>(),
        std::mem::size_of::<GpuPipelineStatistics>(),
        std::mem::size_of::<GpuPassPipelineStatistics>(),
        std::mem::size_of::<GpuPipelineStatisticsFrameResult>(),
    ];

    assert!(projected_type_sizes.iter().all(|size| *size > 0));
    assert!(DEFAULT_GPU_TIMER_MAX_PASSES > 0);
    assert!(DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES > 0);
    assert_eq!(GpuTimerFrameStatus::Pending, GpuTimerFrameStatus::Pending);
}
