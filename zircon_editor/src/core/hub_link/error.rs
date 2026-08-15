use std::io;
use std::path::PathBuf;

use thiserror::Error;
use zircon_runtime_interface::hub_protocol::HubEditorFocusSignalPathError;

#[derive(Debug, Error)]
pub enum HubFocusSignalError {
    #[error(transparent)]
    Path(#[from] HubEditorFocusSignalPathError),
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
}
