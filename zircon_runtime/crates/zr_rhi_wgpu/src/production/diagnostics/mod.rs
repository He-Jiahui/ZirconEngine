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
