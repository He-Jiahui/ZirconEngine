use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub use crate::core::process::ProcessTreeTerminationError as ExportProcessTerminationError;

#[derive(Debug, Error)]
pub enum ExportProcessError {
    #[error(
        "{operation} for {label}{stream_suffix}{path_suffix}: {source}",
        stream_suffix = stream.map(|stream| format!(" ({stream})")).unwrap_or_default(),
        path_suffix = path.as_ref().map(|path| format!(" at {}", path.display())).unwrap_or_default()
    )]
    Io {
        operation: &'static str,
        label: String,
        stream: Option<&'static str>,
        path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },
    #[error("{label} was cancelled before process launch")]
    CancelledBeforeLaunch { label: String },
    #[error("{label} process termination failed: {diagnostic}: {source}")]
    TerminationFailed {
        label: String,
        diagnostic: String,
        #[source]
        source: Box<ExportProcessTerminationError>,
    },
    #[error("{source}; cleanup: {cleanup_diagnostic}")]
    Cleanup {
        #[source]
        source: Box<ExportProcessError>,
        cleanup_diagnostic: String,
        cleanup_error: Option<Box<ExportProcessTerminationError>>,
    },
}

impl ExportProcessError {
    pub(in crate::ui::host) fn io(
        operation: &'static str,
        label: impl Into<String>,
        stream: Option<&'static str>,
        path: Option<PathBuf>,
        source: io::Error,
    ) -> Self {
        Self::Io {
            operation,
            label: label.into(),
            stream,
            path,
            source,
        }
    }

    pub(in crate::ui::host) fn with_cleanup(
        self,
        cleanup_diagnostic: String,
        cleanup_error: Option<ExportProcessTerminationError>,
    ) -> Self {
        Self::Cleanup {
            source: Box::new(self),
            cleanup_diagnostic,
            cleanup_error: cleanup_error.map(Box::new),
        }
    }
}

#[cfg(test)]
#[path = "tests/error.rs"]
mod tests;
