use thiserror::Error;

use super::RenderCapabilityMismatchDetail;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RenderFrameworkError {
    #[error("render framework viewport `{viewport}` does not exist")]
    UnknownViewport { viewport: u64 },
    #[error("render framework pipeline `{pipeline}` does not exist")]
    UnknownPipeline { pipeline: u64 },
    #[error("render framework pipeline `{pipeline}` failed graph validation: {message}")]
    GraphCompileFailure { pipeline: u64, message: String },
    #[error("render framework pipeline `{pipeline}` is not compatible with backend capabilities: {reason}")]
    CapabilityMismatch {
        pipeline: u64,
        reason: String,
        missing: Vec<RenderCapabilityMismatchDetail>,
    },
    #[error("render backend error: {0}")]
    Backend(String),
}
