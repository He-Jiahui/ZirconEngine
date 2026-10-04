mod catalog;
mod metadata;
mod report;
mod snapshot_metadata;
mod source_path;

pub(crate) use catalog::AutosaveRecoveryCatalog;
use metadata::{AutosaveRecoveryMetadata, RECOVERY_METADATA_FILE_NAME};
pub use report::{
    AutosaveRecoveryCatalogDiagnostic, AutosaveRecoveryCatalogDiagnosticKind,
    AutosaveRecoveryCatalogReport,
};
pub(crate) use snapshot_metadata::{
    snapshot_metadata_path, snapshot_metadata_sequence, AutosaveSnapshotMetadata,
};
pub use source_path::AutosaveSourcePath;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
