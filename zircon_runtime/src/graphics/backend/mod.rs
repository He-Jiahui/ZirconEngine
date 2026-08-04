//! GPU device and surface management.

mod render_backend;

#[cfg(test)]
pub(crate) use render_backend::read_buffer_f32x4;
#[cfg(test)]
pub(crate) use render_backend::read_texture_rgba16float_3d;
#[cfg(test)]
pub(crate) use render_backend::RenderBackendConfig;
#[allow(unused_imports)]
pub(crate) use render_backend::{
    read_buffer_bytes, read_buffer_f32x4_array_bytes, read_buffer_sh9_f32x4_bytes,
    read_ibl_bake_artifact_wgpu_sections, read_texture_rgba16float_cube_mip_chain,
    read_texture_rgba16float_region, BufferByteReadback, IblBakeArtifactWgpuReadbackResources,
    Rgba16FloatTextureRegionReadback,
};
pub(crate) use render_backend::{
    read_texture_rgba, GraphicsDebuggerCaptureStop, OffscreenTarget, RenderBackend, ViewportSurface,
};
pub(crate) use render_backend::{
    GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
    GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
    GpuPipelineStatisticsTimer, GpuTimerFrameResult, DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES,
    DEFAULT_GPU_TIMER_MAX_PASSES,
};
pub(crate) use render_backend::{GpuReadbackQueue, ReadbackPollStats};

#[cfg(test)]
mod tests {
    use super::{
        GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
        GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
        GpuPipelineStatisticsTimer, GpuTimerFrameResult,
        DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
    };

    #[test]
    fn gpu_timer_contract_is_available_from_the_backend_root() {
        let projected_type_sizes = [
            std::mem::size_of::<GpuPassTimer>(),
            std::mem::size_of::<GpuPassTimestampScope>(),
            std::mem::size_of::<GpuPassTiming>(),
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
    }
}
