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
mod tests {
    use super::*;

    #[test]
    fn cleanup_retains_typed_termination_error() {
        let error = ExportProcessError::io(
            "read process output",
            "typed cleanup test",
            Some("stdout"),
            None,
            io::Error::new(io::ErrorKind::UnexpectedEof, "primary source"),
        )
        .with_cleanup(
            "termination command could not start".to_string(),
            Some(ExportProcessTerminationError::CommandSpawn {
                program: "taskkill",
                source: io::Error::new(io::ErrorKind::PermissionDenied, "cleanup source"),
            }),
        );

        match error {
            ExportProcessError::Cleanup {
                source,
                cleanup_error: Some(cleanup_error),
                ..
            } => {
                assert!(matches!(*source, ExportProcessError::Io { .. }));
                assert!(matches!(
                    *cleanup_error,
                    ExportProcessTerminationError::CommandSpawn {
                        program: "taskkill",
                        source,
                    } if source.kind() == io::ErrorKind::PermissionDenied
                ));
            }
            other => panic!("expected typed cleanup error, got {other:?}"),
        }
    }
}
