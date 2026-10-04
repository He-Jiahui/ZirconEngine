//! GPU device and surface management.

mod render_backend;

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
pub(crate) use render_backend::ReadbackPollStats;
#[cfg(test)]
pub(crate) use render_backend::RenderBackendConfig;
pub(in crate::graphics) use render_backend::{
    begin_source_cubemap_wgpu_readback, request_source_cubemap_wgpu_readback_batch,
    SourceCubemapWgpuPendingReadback, SourceCubemapWgpuReadback,
};
#[cfg(test)]
pub(crate) use render_backend::{
    read_buffer_bytes, read_buffer_f32x4_array_bytes, read_buffer_sh9_f32x4_bytes,
    BufferByteReadback,
};
#[cfg(test)]
pub(crate) use render_backend::{
    read_texture_rgba16float_cube_mip_chain, read_texture_rgba16float_region,
    Rgba16FloatTextureRegionReadback,
};
pub(crate) use render_backend::{
    request_ibl_bake_artifact_wgpu_readback, IblBakeArtifactWgpuPendingReadback,
    IblBakeArtifactWgpuReadbackResources,
};
pub(crate) use render_backend::{
    GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
    GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
    GpuPipelineStatisticsTimer, GpuTimerFrameObservation, GpuTimerFrameResult, GpuTimerFrameStatus,
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
};
pub(crate) use render_backend::{
    GraphicsDebuggerCaptureStop, OffscreenTarget, RenderBackend, ViewportSurface,
    ViewportSurfaceFrameAcquire, ViewportSurfacePresentFailure, ViewportSurfacePresentOutcome,
};
pub use render_backend::{NeutralMvpCaptureError, NeutralMvpRenderer};
pub(crate) use render_backend::{
    ProductDiagnosticQueryFrameScope, ProductDiagnosticReadbackFrameScope,
};
pub(crate) use render_backend::{
    SystemTextureGenerationLease, SystemTextureGenerationStartupReport,
    SystemTexturePayloadCacheState,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
