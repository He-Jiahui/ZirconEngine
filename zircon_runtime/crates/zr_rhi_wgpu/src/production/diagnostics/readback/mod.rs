//! 读回子模块共享设备唯一轮询边界；帧请求、映射和有界投递按票据连接。
//! 这里的所有权与提交时序由同一设备代际统一管理。
//! Submission-bound WGPU staging readback.
//!
//! This folder owns the native staging-copy and map lifecycle. It is separate
//! from query planning so buffer and texture diagnostics can share the same
//! receipt, quota, and device-poll boundary without making `device.rs` a
//! second readback owner.

#[path = "batch.rs"]
mod batch;
#[path = "../../readback/completion_order.rs"]
pub(crate) mod completion_order;
#[path = "../../readback/delivery.rs"]
mod delivery;
#[path = "layout.rs"]
mod layout;
#[path = "metrics.rs"]
mod metrics;
#[path = "request.rs"]
mod request;
#[path = "service.rs"]
mod service;

#[cfg(test)]
#[path = "../../readback/tests/cases.rs"]
mod tests;

pub(crate) use batch::DiagnosticReadbackBatch;
pub use delivery::WgpuDiagnosticReadbackDelivery;
pub(crate) use layout::{DiagnosticTextureMipChainReadbackLayout, DiagnosticTextureReadbackLayout};
pub use metrics::{WgpuDiagnosticReadbackMetricsDelta, WgpuDiagnosticReadbackMetricsSnapshot};
pub(crate) use request::DiagnosticReadbackSource;
pub(crate) use service::WgpuDiagnosticReadbackService;
