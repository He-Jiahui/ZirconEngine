#[path = "config.rs"]
mod config;
#[path = "gpu_pass_timer/mod.rs"]
mod gpu_pass_timer;
#[path = "capture.rs"]
mod graphics_debugger_capture;
#[path = "neutral_mvp_renderer.rs"]
mod neutral_mvp_renderer;
#[path = "offscreen_target.rs"]
mod offscreen_target;
#[path = "offscreen_target_construct/mod.rs"]
mod offscreen_target_construct;
#[path = "../product_diagnostic_delivery_router.rs"]
mod product_diagnostic_delivery_router;
#[cfg(test)]
#[path = "tests/read_buffer_bytes.rs"]
mod read_buffer_bytes;
#[cfg(test)]
#[path = "tests/read_buffer_f32x4.rs"]
mod read_buffer_f32x4;
#[path = "read_ibl_bake_artifact_sections.rs"]
mod read_ibl_bake_artifact_sections;
#[path = "read_source_cubemap.rs"]
mod read_source_cubemap;
#[cfg(test)]
#[path = "tests/read_texture_rgba.rs"]
mod read_texture_rgba;
#[cfg(test)]
#[path = "tests/read_texture_rgba16float_3d.rs"]
mod read_texture_rgba16float_3d;
#[cfg(test)]
#[path = "../tests/read_texture_rgba16float_region.rs"]
mod read_texture_rgba16float_region;
#[path = "backend.rs"]
mod render_backend;
#[path = "render_backend_diagnostics.rs"]
mod render_backend_diagnostics;
#[path = "render_backend_new_offscreen.rs"]
mod render_backend_new_offscreen;
#[path = "render_backend_submission.rs"]
mod render_backend_submission;
#[cfg(test)]
#[path = "tests/renderdoc_capture_file_path.rs"]
mod renderdoc_capture_file_path;
#[path = "request_device.rs"]
mod request_device;
#[path = "select_adapter.rs"]
mod select_adapter;
#[path = "textures.rs"]
mod system_texture_generation_owner;
#[path = "ui_surface_diagnostic_complete.rs"]
mod ui_surface_diagnostic_completion;
#[path = "viewport_surface.rs"]
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
#[path = "tests/mod_neutral_mvp_renderer.rs"]
mod neutral_mvp_renderer_tests;
