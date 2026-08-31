use std::io;
use std::path::PathBuf;

use thiserror::Error;
use zircon_runtime_interface::hub_protocol::{
    HubEditorFocusSignalError, HubEditorFocusSignalPathError,
};

#[derive(Debug, Error)]
pub enum HubFocusSignalError {
    #[error(transparent)]
    Path(#[from] HubEditorFocusSignalPathError),
    #[error(transparent)]
    Signal(#[from] HubEditorFocusSignalError),
    #[error("failed to {operation} focus signal `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to encode focus signal `{path}`: {source}")]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("focus signal `{path}` is malformed: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "focus signal `{path}` targets editor instance `{actual_instance_id}`, not `{expected_instance_id}`"
    )]
    TargetMismatch {
        path: PathBuf,
        expected_instance_id: String,
        actual_instance_id: String,
    },
    #[error("focus target editor instance `{instance_id}` is not Ready (lifecycle `{lifecycle}`)")]
    TargetNotReady {
        instance_id: String,
        lifecycle: &'static str,
    },
    #[error(
        "focus signal `{path}` targets session generation `{actual_generation}`, not `{expected_generation}`"
    )]
    TargetGenerationMismatch {
        path: PathBuf,
        expected_generation: u64,
        actual_generation: u64,
    },
    #[error("focus signal `{path}` exceeds the bounded mailbox byte limit")]
    RequestTooLarge { path: PathBuf },
    #[error("failed to read the system clock for a focus request: {source}")]
    Clock {
        #[source]
        source: std::time::SystemTimeError,
    },
    #[error("focus request deadline overflowed")]
    DeadlineOverflow,
}
