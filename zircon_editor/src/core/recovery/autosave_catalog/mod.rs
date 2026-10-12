#[path = "catalog.rs"]
mod catalog;
#[path = "metadata.rs"]
mod metadata;
#[path = "report.rs"]
mod report;
#[path = "snapshot_metadata.rs"]
mod snapshot_metadata;
#[path = "source_path.rs"]
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
