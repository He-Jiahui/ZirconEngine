mod config;
mod gpu_pass_timer;
mod graphics_debugger_capture;
mod neutral_mvp_renderer;
mod offscreen_target;
mod offscreen_target_construct;
mod product_diagnostic_delivery_router;
#[cfg(test)]
#[path = "tests/read_buffer_bytes.rs"]
mod read_buffer_bytes;
#[cfg(test)]
#[path = "tests/read_buffer_f32x4.rs"]
mod read_buffer_f32x4;
mod read_ibl_bake_artifact_sections;
mod read_source_cubemap;
#[cfg(test)]
#[path = "tests/read_texture_rgba.rs"]
mod read_texture_rgba;
#[cfg(test)]
#[path = "tests/read_texture_rgba16float_3d.rs"]
mod read_texture_rgba16float_3d;
#[cfg(test)]
#[path = "tests/read_texture_rgba16float_region.rs"]
mod read_texture_rgba16float_region;
mod render_backend;
mod render_backend_diagnostics;
mod render_backend_new_offscreen;
mod render_backend_submission;
#[cfg(test)]
#[path = "tests/renderdoc_capture_file_path.rs"]
mod renderdoc_capture_file_path;
mod request_device;
mod select_adapter;
mod system_texture_generation_owner;
mod viewport_surface;

#[cfg(test)]
pub(crate) use config::RenderBackendConfig;
pub(crate) use gpu_pass_timer::{
    GpuPassPipelineStatistics, GpuPassTimer, GpuPassTimestampScope, GpuPassTiming,
    GpuPipelineStatistics, GpuPipelineStatisticsFrameResult, GpuPipelineStatisticsScope,
    GpuPipelineStatisticsTimer, GpuTimerFrameObservation, GpuTimerFrameResult, GpuTimerFrameStatus,
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, DEFAULT_GPU_TIMER_MAX_PASSES,
    GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES, GPU_TIMESTAMP_REQUIRED_FEATURES,
};
pub(crate) use graphics_debugger_capture::GraphicsDebuggerCaptureStop;
pub use neutral_mvp_renderer::{NeutralMvpCaptureError, NeutralMvpRenderer};
pub(crate) use offscreen_target::OffscreenTarget;
#[cfg(test)]
pub(crate) use read_buffer_bytes::{
    read_buffer_bytes, read_buffer_f32x4_array_bytes, read_buffer_sh9_f32x4_bytes,
    BufferByteReadback,
};
#[cfg(test)]
pub(crate) use read_buffer_f32x4::read_buffer_f32x4;
#[cfg(test)]
pub(crate) use read_ibl_bake_artifact_sections::read_ibl_bake_artifact_wgpu_sections;
pub(crate) use read_ibl_bake_artifact_sections::{
    request_ibl_bake_artifact_wgpu_readback, IblBakeArtifactWgpuPendingReadback,
    IblBakeArtifactWgpuReadbackResources,
};
pub(crate) use read_source_cubemap::{
    begin_source_cubemap_wgpu_readback, request_source_cubemap_wgpu_readback_batch,
    SourceCubemapWgpuPendingReadback, SourceCubemapWgpuReadback, SourceCubemapWgpuReadbackBatch,
};
#[cfg(test)]
pub(crate) use read_texture_rgba::read_texture_rgba;
#[cfg(test)]
pub(crate) use read_texture_rgba16float_3d::read_texture_rgba16float_3d;
#[cfg(test)]
pub(crate) use read_texture_rgba16float_region::{
    read_texture_rgba16float_cube_mip_chain, read_texture_rgba16float_region,
    Rgba16FloatTextureRegionReadback,
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
#[path = "tests/mod_neutral_mvp_renderer_tests.rs"]
mod neutral_mvp_renderer_tests;
