use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type ExportValidateResult<T> = std::result::Result<T, ExportValidateError>;

#[derive(Debug, Error)]
pub enum ExportValidateError {
    #[error("{0}")]
    Usage(String),
    #[error("failed to encode export validate report: {source}")]
    EncodeReport {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create export validate report directory {}: {source}", path.display())]
    CreateReportDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write export validate report {}: {source}", path.display())]
    WriteReport {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
