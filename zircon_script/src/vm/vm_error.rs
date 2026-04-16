use thiserror::Error;

#[derive(Debug, Error)]
pub enum VmError {
    #[error("vm backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("plugin slot missing: {0}")]
    MissingSlot(u64),
    #[error("plugin operation failed: {0}")]
    Operation(String),
    #[error("package parse failed: {0}")]
    Parse(String),
}
