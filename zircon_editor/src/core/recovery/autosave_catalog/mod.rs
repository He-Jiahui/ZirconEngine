mod catalog;
mod metadata;
mod source_path;

pub(crate) use catalog::AutosaveRecoveryCatalog;
use metadata::{AutosaveRecoveryMetadata, RECOVERY_METADATA_FILE_NAME};
pub use source_path::AutosaveSourcePath;

#[cfg(test)]
mod tests;
