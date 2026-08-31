use std::time::Duration;

use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ConfigManagerError {
    #[error("configuration runtime is no longer available")]
    RuntimeUnavailable,
    #[error("configuration persistence failed for {path}: {reason}")]
    Persistence { path: String, reason: String },
    #[error("configuration flush timed out for {path} after {timeout:?}")]
    FlushTimedOut { path: String, timeout: Duration },
}
