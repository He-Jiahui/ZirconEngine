//! Submission-bound WGPU staging readback.
//!
//! This folder owns the native staging-copy and map lifecycle. It is separate
//! from query planning so buffer and texture diagnostics can share the same
//! receipt, quota, and device-poll boundary without making `device.rs` a
//! second readback owner.

mod batch;
pub(crate) mod completion_order;
mod delivery;
mod layout;
mod metrics;
mod request;
mod service;

#[cfg(test)]
mod tests;

pub(crate) use batch::DiagnosticReadbackBatch;
pub use delivery::WgpuDiagnosticReadbackDelivery;
pub(crate) use layout::{DiagnosticTextureMipChainReadbackLayout, DiagnosticTextureReadbackLayout};
pub use metrics::{WgpuDiagnosticReadbackMetricsDelta, WgpuDiagnosticReadbackMetricsSnapshot};
pub(crate) use request::DiagnosticReadbackSource;
pub(crate) use service::WgpuDiagnosticReadbackService;
