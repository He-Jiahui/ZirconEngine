mod binding;
mod buffer_upload_batch;
mod command_encoder;
mod command_list;
mod command_submission;
mod device;
mod diagnostics;
mod fault_terminal;
mod mvp;
mod mvp_surface_triangle;
mod mvp_triangle_pipeline;
mod pipeline;
mod registry;
mod resource_upload_batch;
mod submission;
mod submission_metrics;
mod surface;
mod surface_bootstrap;
mod surface_bootstrap_adapter;
mod translate;
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
pub use upload_batch::{WgpuTextureUpload, WgpuTextureUploadBatch};

#[cfg(test)]
mod tests;
