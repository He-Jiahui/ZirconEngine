//! 生产适配层收拢资源表、命令编码、提交、表面和诊断入口。
//! 设备管理中立票据和故障时间线；过渡期场景和界面仍可借同设备原生句柄录制，再移交提交包。
#[path = "binding.rs"]
mod binding;
#[path = "buffer_upload_batch.rs"]
mod buffer_upload_batch;
#[path = "command_encoder.rs"]
mod command_encoder;
#[path = "command_list.rs"]
mod command_list;
#[path = "command_submission.rs"]
mod command_submission;
#[path = "device.rs"]
mod device;
#[path = "diagnostics/mod.rs"]
mod diagnostics;
#[path = "fault_terminal.rs"]
mod fault_terminal;
#[path = "mvp.rs"]
mod mvp;
#[path = "mvp_surface_triangle.rs"]
mod mvp_surface_triangle;
#[path = "mvp_triangle_pipeline.rs"]
mod mvp_triangle_pipeline;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "registry.rs"]
mod registry;
#[path = "resource_upload_batch.rs"]
mod resource_upload_batch;
#[path = "submission.rs"]
mod submission;
#[path = "submission_metrics.rs"]
mod submission_metrics;
#[path = "surface.rs"]
mod surface;
#[path = "surface_bootstrap.rs"]
mod surface_bootstrap;
#[path = "surface_bootstrap_adapter.rs"]
mod surface_bootstrap_adapter;
#[path = "translate.rs"]
mod translate;
#[path = "upload_batch.rs"]
mod upload_batch;

pub use buffer_upload_batch::{WgpuBufferUpload, WgpuBufferUploadBatch};
pub(crate) use command_list::WgpuCommandList;
pub(crate) use command_submission::encode_command_list;
pub use device::{
    WgpuNativeDiagnosticReadbackFrame, WgpuNativeRecorderLease, WgpuNativeSubmissionPacket,
    WgpuNativeSurfaceFrameTarget, WgpuRenderDevice, WgpuRenderDeviceContext,
};
pub use diagnostics::{
    WgpuDiagnosticQueryDelivery, WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticQueryRecorder,
};
pub use diagnostics::{
    WgpuDiagnosticReadbackDelivery, WgpuDiagnosticReadbackMetricsDelta,
    WgpuDiagnosticReadbackMetricsSnapshot,
};
pub use mvp::WgpuMvpOffscreenTriangle;
pub use mvp_surface_triangle::WgpuMvpSurfaceTriangle;
pub(crate) use registry::WgpuResourceRegistry;
pub use resource_upload_batch::WgpuResourceUploadBatch;
pub(crate) use submission::WgpuSubmissionService;
pub use submission_metrics::{WgpuSubmissionMetricsDelta, WgpuSubmissionMetricsSnapshot};
pub(crate) use surface::WgpuSurfaceService;
pub use surface_bootstrap::WgpuSurfaceBootstrap;
pub use surface_bootstrap_adapter::WgpuSurfaceAdapterBootstrap;
pub use upload_batch::{WgpuTextureUpload, WgpuTextureUploadBatch, WgpuTextureUploadPayload};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
