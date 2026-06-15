use std::{fmt, path::PathBuf};

use crate::asset::pack::ZrPackError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZrPackDeltaInstallError {
    ReadFailed {
        path: PathBuf,
        error: String,
    },
    WriteFailed {
        path: PathBuf,
        error: String,
    },
    RenameFailed {
        source: PathBuf,
        destination: PathBuf,
        error: String,
    },
    ReceiptEncode(String),
    ReceiptDecode(String),
    ReceiptReportMismatch(String),
    Pack(ZrPackError),
}

impl fmt::Display for ZrPackDeltaInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed { path, error } => {
                write!(
                    formatter,
                    "failed to read pack file {}: {error}",
                    path.display()
                )
            }
            Self::WriteFailed { path, error } => {
                write!(
                    formatter,
                    "failed to write file {}: {error}",
                    path.display()
                )
            }
            Self::RenameFailed {
                source,
                destination,
                error,
            } => write!(
                formatter,
                "failed to move pack file {} to {}: {error}",
                source.display(),
                destination.display()
            ),
            Self::ReceiptEncode(error) => {
                write!(
                    formatter,
                    "failed to encode zrpack install receipt: {error}"
                )
            }
            Self::ReceiptDecode(error) => {
                write!(
                    formatter,
                    "failed to decode zrpack install receipt: {error}"
                )
            }
            Self::ReceiptReportMismatch(message) => {
                write!(
                    formatter,
                    "zrpack install receipt reports are inconsistent: {message}"
                )
            }
            Self::Pack(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ZrPackDeltaInstallError {}

impl From<ZrPackError> for ZrPackDeltaInstallError {
    fn from(error: ZrPackError) -> Self {
        Self::Pack(error)
    }
}
