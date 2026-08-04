use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::SessionLockRecord;

#[derive(Debug, Error)]
pub enum SessionGuardError {
    #[error("session lock already exists at `{path}`")]
    AlreadyHeld {
        path: PathBuf,
        record: Option<SessionLockRecord>,
    },
    #[error("session lock at `{path}` is no longer owned by this editor instance")]
    OwnershipLost { path: PathBuf },
    #[error("invalid session lock at `{path}`: {message}")]
    InvalidRecord { path: PathBuf, message: String },
    #[error("failed to {operation} at `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("system clock is before the Unix epoch")]
    ClockBeforeUnixEpoch,
    #[error("project session ownership is not implemented for this platform")]
    PlatformUnsupported,
}
