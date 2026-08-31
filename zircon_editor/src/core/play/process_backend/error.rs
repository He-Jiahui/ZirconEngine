use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessPlayBackendInstallError {
    #[error("failed to resolve the current editor executable: {source}")]
    CurrentEditorExecutable {
        #[source]
        source: io::Error,
    },
    #[error("the current editor executable has no parent installation directory")]
    MissingEditorInstallDirectory,
    #[error("failed to resolve the current editor installation directory: {source}")]
    ResolveEditorInstallDirectory {
        #[source]
        source: io::Error,
    },
    #[error("failed to resolve the sibling runtime executable: {source}")]
    ResolveRuntimeExecutable {
        #[source]
        source: io::Error,
    },
}
