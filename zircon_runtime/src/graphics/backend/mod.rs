//! GPU device and surface management.

mod render_backend;

pub(crate) use render_backend::ReadbackPollStats;
#[cfg(test)]
pub(crate) use render_backend::RenderBackendConfig;
#[cfg(test)]
pub(crate) use render_backend::configure_renderdoc_capture_file_path_template;
#[cfg(test)]
pub(crate) use render_backend::read_buffer_f32x4;
#[cfg(test)]
pub(crate) use render_backend::read_ibl_bake_artifact_wgpu_sections;
#[cfg(test)]
pub(crate) use render_backend::read_texture_rgba;
#[cfg(test)]
pub(crate) use render_backend::read_texture_rgba16float_3d;
#[cfg(test)]
pub(crate) use render_backend::{
    BufferByteReadback, read_buffer_bytes, read_buffer_f32x4_array_bytes,
    read_buffer_sh9_f32x4_bytes,
};
pub(crate) use render_backend::{
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
    GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
    GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
    GpuPipelineStatisticsTimer, GpuTimerFrameObservation, GpuTimerFrameResult, GpuTimerFrameStatus,
};
pub(crate) use render_backend::{
    GraphicsDebuggerCaptureStop, OffscreenTarget, RenderBackend, ViewportSurface,
    ViewportSurfaceFrameAcquire, ViewportSurfacePresentFailure, ViewportSurfacePresentOutcome,
};
#[allow(unused_imports)]
pub(crate) use render_backend::{
    IblBakeArtifactWgpuPendingReadback, IblBakeArtifactWgpuReadbackResources,
    request_ibl_bake_artifact_wgpu_readback,
};
pub use render_backend::{NeutralMvpCaptureError, NeutralMvpRenderer};
pub(crate) use render_backend::{
    ProductDiagnosticQueryFrameScope, ProductDiagnosticReadbackFrameScope,
};
#[cfg(test)]
pub(crate) use render_backend::{
    Rgba16FloatTextureRegionReadback, read_texture_rgba16float_cube_mip_chain,
    read_texture_rgba16float_region,
};
pub(crate) use render_backend::{
    SystemTextureGenerationLease, SystemTextureGenerationStartupReport,
    SystemTexturePayloadCacheState,
};

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
        GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
        GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
        GpuPipelineStatisticsTimer, GpuTimerFrameObservation, GpuTimerFrameResult,
        GpuTimerFrameStatus,
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
}
