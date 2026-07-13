use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::super::export_process_support::ExportProcessError;

#[derive(Debug, Error)]
pub enum NativeDynamicPreparationError {
    #[error(
        "{operation} for native dynamic package {package_id}{path_suffix}: {source}",
        path_suffix = path.as_ref().map(|path| format!(" at {}", path.display())).unwrap_or_default()
    )]
    Io {
        operation: &'static str,
        package_id: String,
        path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Process(#[from] ExportProcessError),
    #[error("failed to remove native dynamic temporary directory {}: {source}", path.display())]
    Cleanup {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("native dynamic preparation failed: {source}; cleanup also failed: {cleanup}")]
    PreparationFailedWithCleanup {
        #[source]
        source: Box<NativeDynamicPreparationError>,
        cleanup: Box<NativeDynamicPreparationError>,
    },
    #[error(
        "native dynamic cleanup failed at {source}; additional cleanup failures: {additional:?}"
    )]
    CleanupBatch {
        #[source]
        source: NativeDynamicCleanupError,
        additional: Vec<NativeDynamicCleanupError>,
    },
}

#[derive(Debug, Error)]
#[error("failed to remove native dynamic temporary directory {}: {source}", path.display())]
pub struct NativeDynamicCleanupError {
    pub(super) path: PathBuf,
    #[source]
    pub(super) source: io::Error,
}

impl NativeDynamicPreparationError {
    pub(super) fn io(
        operation: &'static str,
        package_id: impl Into<String>,
        path: Option<PathBuf>,
        source: io::Error,
    ) -> Self {
        Self::Io {
            operation,
            package_id: package_id.into(),
            path,
            source,
        }
    }

    pub(super) fn cleanup(path: PathBuf, source: io::Error) -> Self {
        Self::Cleanup { path, source }
    }

    pub(super) fn cleanup_batch(mut errors: Vec<NativeDynamicCleanupError>) -> Self {
        debug_assert!(!errors.is_empty());
        let source = errors.remove(0);
        Self::CleanupBatch {
            source,
            additional: errors,
        }
    }

    pub(super) fn with_cleanup_failure(self, cleanup: NativeDynamicPreparationError) -> Self {
        Self::PreparationFailedWithCleanup {
            source: Box::new(self),
            cleanup: Box::new(cleanup),
        }
    }
}

impl NativeDynamicCleanupError {
    pub(super) fn new(path: PathBuf, source: io::Error) -> Self {
        Self { path, source }
    }
}
