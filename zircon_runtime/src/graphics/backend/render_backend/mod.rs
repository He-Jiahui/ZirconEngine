mod config;
mod gpu_pass_timer;
mod graphics_debugger_capture;
mod neutral_mvp_renderer;
mod offscreen_target;
mod offscreen_target_construct;
mod product_diagnostic_delivery_router;
#[cfg(test)]
mod read_buffer_bytes;
#[cfg(test)]
mod read_buffer_f32x4;
mod read_ibl_bake_artifact_sections;
mod read_source_cubemap;
#[cfg(test)]
mod read_texture_rgba;
#[cfg(test)]
mod read_texture_rgba16float_3d;
#[cfg(test)]
mod read_texture_rgba16float_region;
mod render_backend;
mod render_backend_diagnostics;
mod render_backend_new_offscreen;
mod render_backend_submission;
#[cfg(test)]
mod renderdoc_capture_file_path;
mod request_device;
mod select_adapter;
mod system_texture_generation_owner;
mod viewport_surface;

#[cfg(test)]
pub(crate) use config::RenderBackendConfig;
pub(crate) use gpu_pass_timer::{
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
    GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES, GPU_TIMESTAMP_REQUIRED_FEATURES,
    GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
    GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
    GpuPipelineStatisticsTimer, GpuTimerFrameObservation, GpuTimerFrameResult, GpuTimerFrameStatus,
};
pub(crate) use graphics_debugger_capture::GraphicsDebuggerCaptureStop;
pub use neutral_mvp_renderer::{NeutralMvpCaptureError, NeutralMvpRenderer};
pub(crate) use offscreen_target::OffscreenTarget;
#[cfg(test)]
pub(crate) use read_buffer_bytes::{
    BufferByteReadback, read_buffer_bytes, read_buffer_f32x4_array_bytes,
    read_buffer_sh9_f32x4_bytes,
};
#[cfg(test)]
pub(crate) use read_buffer_f32x4::read_buffer_f32x4;
#[cfg(test)]
pub(crate) use read_ibl_bake_artifact_sections::read_ibl_bake_artifact_wgpu_sections;
pub(crate) use read_ibl_bake_artifact_sections::{
    IblBakeArtifactWgpuPendingReadback, IblBakeArtifactWgpuReadbackResources,
    request_ibl_bake_artifact_wgpu_readback,
};
pub(crate) use read_source_cubemap::{
    SourceCubemapWgpuPendingReadback, SourceCubemapWgpuReadback, SourceCubemapWgpuReadbackBatch,
    begin_source_cubemap_wgpu_readback, request_source_cubemap_wgpu_readback_batch,
};
#[cfg(test)]
pub(crate) use read_texture_rgba::read_texture_rgba;
#[cfg(test)]
pub(crate) use read_texture_rgba16float_3d::read_texture_rgba16float_3d;
#[cfg(test)]
pub(crate) use read_texture_rgba16float_region::{
    Rgba16FloatTextureRegionReadback, read_texture_rgba16float_cube_mip_chain,
    read_texture_rgba16float_region,
};
pub(crate) use render_backend::RenderBackend;
pub(crate) use render_backend_diagnostics::{
    ProductDiagnosticQueryFrameScope, ProductDiagnosticReadbackFrameScope,
};
#[cfg(test)]
pub(crate) use renderdoc_capture_file_path::configure_renderdoc_capture_file_path_template;
use select_adapter::select_offscreen_adapter;
pub(crate) use system_texture_generation_owner::{
    SystemTextureGenerationLease, SystemTextureGenerationStartupReport,
    SystemTexturePayloadCacheState,
};
pub(crate) use viewport_surface::{
    ViewportSurface, ViewportSurfaceFrameAcquire, ViewportSurfacePresentFailure,
    ViewportSurfacePresentOutcome,
};
pub(crate) use zr_rhi_wgpu::ReadbackPollStats;

#[cfg(test)]
mod neutral_mvp_renderer_tests {
    #[test]
    fn neutral_mvp_renderer_transfers_native_context_without_a_raw_backend_owner() {
        let source = include_str!("neutral_mvp_renderer.rs");

        assert!(source.contains("WgpuRenderDeviceContext::new("));
        assert!(source.contains("WgpuRenderDevice::new(context, profile)"));
        assert!(source.contains("WgpuMvpOffscreenTriangle::new(&device"));
        assert!(source.contains("self.frame.submit(&self.device)"));
        assert!(!source.contains("RenderBackend::new_offscreen"));
        assert!(!source.contains("queue.submit"));
        assert!(!source.contains("WgpuDeviceErrorSupervisor::install"));
    }
}
