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
}
