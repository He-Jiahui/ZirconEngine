use std::io;
use std::path::PathBuf;

use crate::pack::ZrPackError;
use thiserror::Error;

pub type ExportPackResult<T> = std::result::Result<T, ExportPackError>;

#[derive(Debug, Error)]
pub enum ExportPackError {
    #[error("{0}")]
    Usage(String),
    #[error("failed to read asset pack manifest {}: {source}", path.display())]
    ReadAssetManifest {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to decode asset pack manifest: {source}")]
    DecodeAssetManifest {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create pack directory {}: {source}", path.display())]
    CreatePackDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write pack {}: {source}", path.display())]
    WritePack {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to encode export pack report: {source}")]
    EncodeReport {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create export pack report directory {}: {source}", path.display())]
    CreateReportDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write export pack report {}: {source}", path.display())]
    WriteReport {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read previous pack {}: {source}", path.display())]
    ReadPreviousPack {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read previous zrpack: {source}")]
    ReadPreviousZrPack {
        #[source]
        source: ZrPackError,
    },
    #[error("failed to read newly written zrpack: {source}")]
    ReadNewlyWrittenZrPack {
        #[source]
        source: ZrPackError,
    },
    #[error("failed to write delta zrpack: {source}")]
    WriteDeltaZrPack {
        #[source]
        source: ZrPackError,
    },
    #[error("failed to create delta pack directory {}: {source}", path.display())]
    CreateDeltaPackDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write delta pack {}: {source}", path.display())]
    WriteDeltaPack {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to verify written delta zrpack: {source}")]
    VerifyWrittenDeltaZrPack {
        #[source]
        source: ZrPackError,
    },
    #[error("failed to verify delta asset {asset}: {source}")]
    VerifyDeltaAsset {
        asset: String,
        #[source]
        source: ZrPackError,
    },
    #[error("failed to apply delta pack to previous zrpack: {source}")]
    ApplyDeltaPack {
        #[source]
        source: ZrPackError,
    },
    #[error("delta pack apply verification did not reconstruct target zrpack")]
    DeltaApplyVerificationMismatch,
    #[error("failed to write deterministic comparison pack: {source}")]
    DeterministicComparisonWrite {
        #[source]
        source: ZrPackError,
    },
}
