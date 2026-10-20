//! 此边界把中立设备、提交票据和诊断契约接到 WGPU；实际资源与完成时间线由生产设备持有。
//! 测试用主机镜像只验证描述符与调用顺序，不能用其通过结果证明原生着色器或 GPU 同步正确。
//!
//! `wgpu` capability mapping and native UI presentation support.
//!
//! Current product scene/offscreen ownership remains in `graphics::backend` until the Runtime90
//! product cutover. `production::WgpuRenderDevice` is the generation-qualified neutral RHI owner
//! used to converge that cutover without exposing its native WGPU objects. A retained-UI surface
//! either owns a surface-compatible device for standalone fallback or receives a cloned runtime
//! WGPU context for same-device composition. The deterministic host mirror exists only for RHI
//! contract tests.

#[path = "bind_group_validation.rs"]
mod bind_group_validation;
#[path = "capabilities.rs"]
mod capabilities;
#[cfg(test)]
#[path = "tests/command_validation.rs"]
mod command_validation;
#[cfg(test)]
#[path = "tests/device.rs"]
mod device;
#[path = "device_fault.rs"]
mod device_fault;
#[path = "device_profile.rs"]
mod device_profile;
#[path = "gpu_diagnostic_query_frame.rs"]
mod gpu_diagnostic_query_frame;
#[path = "gpu_pass_timer.rs"]
mod gpu_pass_timer;
#[path = "gpu_pipeline_statistics.rs"]
mod gpu_pipeline_statistics;
#[path = "gpu_readback_queue/mod.rs"]
mod gpu_readback_queue;
#[path = "indirect_validation.rs"]
mod indirect_validation;
#[path = "pipeline_validation.rs"]
mod pipeline_validation;
#[path = "production/mod.rs"]
mod production;
#[path = "render_pass_validation.rs"]
mod render_pass_validation;
#[path = "resource_validation.rs"]
mod resource_validation;
#[path = "texture_copy.rs"]
mod texture_copy;
#[path = "texture_view.rs"]
mod texture_view;
#[path = "ui_surface.rs"]
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
    WgpuTextureUploadPayload,
};
pub use ui_surface::{
    WgpuUiExternalImage, WgpuUiExternalImageAlphaMode, WgpuUiExternalImageCopyReceipt,
    WgpuUiExternalImageCopyTarget, WgpuUiSharedImageRegistry, WgpuUiSurfaceContext,
    WgpuUiSurfaceDiagnosticCompletionOwner, WgpuUiSurfaceDiagnosticReceiptCallback,
    WgpuUiSurfaceExternalImageProvider, WgpuUiSurfacePresenter,
};

#[cfg(test)]
use device::{DeterministicRhiContractCommandList, DeterministicRhiContractDevice};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

#[cfg(target_arch = "wasm32")]
#[path = "canvas_surface.rs"]
mod canvas_surface;
#[cfg(target_arch = "wasm32")]
pub use canvas_surface::{
    register_canvas_surface, registered_canvas_surface_count, CanvasSurfaceRegistration,
};
