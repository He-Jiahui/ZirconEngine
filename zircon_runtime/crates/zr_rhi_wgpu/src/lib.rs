//! `wgpu` capability mapping and native UI presentation support.
//!
//! Current product scene/offscreen ownership remains in `graphics::backend` until the Runtime90
//! product cutover. `production::WgpuRenderDevice` is the generation-qualified neutral RHI owner
//! used to converge that cutover without exposing its native WGPU objects. A retained-UI surface
//! either owns a surface-compatible device for standalone fallback or receives a cloned runtime
//! WGPU context for same-device composition. The deterministic host mirror exists only for RHI
//! contract tests.

mod bind_group_validation;
mod capabilities;
#[cfg(test)]
mod command_validation;
#[cfg(test)]
mod device;
mod device_fault;
mod device_profile;
mod gpu_diagnostic_query_frame;
mod gpu_pass_timer;
mod gpu_pipeline_statistics;
mod gpu_readback_queue;
mod indirect_validation;
mod pipeline_validation;
mod production;
mod render_pass_validation;
mod resource_validation;
mod texture_copy;
mod texture_view;
mod ui_surface;

pub use capabilities::wgpu_backend_caps;
pub use device_fault::WgpuDeviceErrorSupervisor;
pub use device_profile::{
    initial_wgpu_render_device_profile, next_wgpu_device_id, wgpu_adapter_facts,
    wgpu_device_limits, wgpu_device_request, wgpu_features_for_device_request, WgpuDeviceRequest,
    WGPU_BINDLESS_MATERIAL_REQUIRED_FEATURES,
};
pub use gpu_diagnostic_query_frame::{
    GpuDiagnosticQueryFramePlan, GpuDiagnosticQueryFramePlanSnapshot,
};
pub use gpu_pass_timer::{
    GpuPassTimer, GpuPassTimestampScope, GpuPassTiming, GpuTimerFrameObservation,
    GpuTimerFrameResult, GpuTimerFrameStatus, DEFAULT_GPU_TIMER_MAX_PASSES,
    GPU_TIMESTAMP_REQUIRED_FEATURES,
};
pub use gpu_pipeline_statistics::{
    GpuPassPipelineStatistics, GpuPipelineStatistics, GpuPipelineStatisticsFrameResult,
    GpuPipelineStatisticsScope, GpuPipelineStatisticsTimer,
    DEFAULT_GPU_PIPELINE_STATISTICS_MAX_SCOPES, GPU_PIPELINE_STATISTICS_REQUIRED_FEATURES,
};
pub use gpu_readback_queue::{
    GpuReadbackQueue, ReadbackCallback, ReadbackError, ReadbackPollStats, ReadbackTicket,
};
pub use production::{
    WgpuBufferUpload, WgpuBufferUploadBatch, WgpuDiagnosticQueryDelivery,
    WgpuDiagnosticReadbackDelivery, WgpuDiagnosticReadbackMetricsDelta,
    WgpuDiagnosticReadbackMetricsSnapshot, WgpuMvpOffscreenTriangle, WgpuMvpSurfaceTriangle,
    WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticQueryRecorder,
    WgpuNativeDiagnosticReadbackFrame, WgpuNativeRecorderLease, WgpuNativeSubmissionPacket,
    WgpuNativeSurfaceFrameTarget, WgpuRenderDevice, WgpuRenderDeviceContext,
    WgpuResourceUploadBatch, WgpuSubmissionMetricsDelta, WgpuSubmissionMetricsSnapshot,
    WgpuSurfaceAdapterBootstrap, WgpuSurfaceBootstrap, WgpuTextureUpload, WgpuTextureUploadBatch,
};
pub use ui_surface::{
    WgpuUiExternalImage, WgpuUiExternalImageAlphaMode, WgpuUiExternalImageCopyReceipt,
    WgpuUiExternalImageCopyTarget, WgpuUiSharedImageRegistry, WgpuUiSurfaceContext,
    WgpuUiSurfaceExternalImageProvider, WgpuUiSurfacePresenter,
};

#[cfg(test)]
use device::{DeterministicRhiContractCommandList, DeterministicRhiContractDevice};

#[cfg(test)]
mod tests;
