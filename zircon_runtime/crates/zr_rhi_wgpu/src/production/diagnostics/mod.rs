//! 诊断服务拥有原生查询集、解析缓冲、暂存、映射和投递；中立查询规划、接收凭据和预算遵守公共契约。
//! 这里的所有权与提交时序由同一设备代际统一管理。
//! Production WGPU diagnostic readback ownership.
//!
//! Neutral query planning and aggregation live in `zr_rhi`; this module owns
//! only native staging-copy, mapping, and delivery behavior.

mod query;
mod readback;

pub use query::{
    WgpuDiagnosticQueryDelivery, WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticQueryRecorder,
};
pub(crate) use query::{WgpuDiagnosticQueryFrame, WgpuDiagnosticQueryService};
pub(crate) use readback::{
    DiagnosticReadbackBatch, DiagnosticReadbackSource, DiagnosticTextureMipChainReadbackLayout,
    DiagnosticTextureReadbackLayout, WgpuDiagnosticReadbackService,
};
pub use readback::{
    WgpuDiagnosticReadbackDelivery, WgpuDiagnosticReadbackMetricsDelta,
    WgpuDiagnosticReadbackMetricsSnapshot,
};
