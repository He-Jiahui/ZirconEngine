use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::AutosaveSourcePath;
use crate::core::recovery::{
    AutosaveContentDigest, AutosaveDocumentId, AutosaveError, AutosaveExtension,
    AutosaveSnapshotProvenance,
};

const SNAPSHOT_METADATA_VERSION: u32 = 1;
const SNAPSHOT_METADATA_SUFFIX: &str = ".snapshot.json";

/// The immutable commit marker for one recovery payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct AutosaveSnapshotMetadata {
    version: u32,
    document: AutosaveDocumentId,
    source_path: AutosaveSourcePath,
    source_identity: AutosaveContentDigest,
    extension: AutosaveExtension,
    provenance: AutosaveSnapshotProvenance,
    committed_checksum: AutosaveContentDigest,
}

impl AutosaveSnapshotMetadata {
    pub(super) fn new(
        document: AutosaveDocumentId,
        source_path: AutosaveSourcePath,
        extension: AutosaveExtension,
        provenance: AutosaveSnapshotProvenance,
        committed_checksum: AutosaveContentDigest,
    ) -> Self {
        Self {
            version: SNAPSHOT_METADATA_VERSION,
            source_identity: source_identity(&source_path),
            document,
            source_path,
            extension,
            provenance,
            committed_checksum,
        }
    }

    pub(super) fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub(super) fn source_path(&self) -> &AutosaveSourcePath {
        &self.source_path
    }

    pub(super) fn source_identity(&self) -> &AutosaveContentDigest {
        &self.source_identity
    }

    pub(super) fn extension(&self) -> &AutosaveExtension {
        &self.extension
    }

    pub(super) fn provenance(&self) -> &AutosaveSnapshotProvenance {
        &self.provenance
    }

    pub(super) fn committed_checksum(&self) -> &AutosaveContentDigest {
        &self.committed_checksum
    }

    pub(super) fn encode(&self, path: &Path) -> Result<Vec<u8>, AutosaveError> {
        serde_json::to_vec(self).map_err(|error| AutosaveError::InvalidRecoveryMetadata {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    }

    pub(super) fn decode(path: &Path, bytes: &[u8]) -> Result<Self, AutosaveError> {
        let metadata = serde_json::from_slice::<Self>(bytes).map_err(|error| {
            AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: error.to_string(),
            }
        })?;
        if metadata.version != SNAPSHOT_METADATA_VERSION {
            return Err(AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "unsupported snapshot metadata version".to_string(),
            });
        }
        let source_path =
            AutosaveSourcePath::parse(metadata.source_path.as_path()).map_err(|_| {
                AutosaveError::InvalidRecoveryMetadata {
                    path: path.to_path_buf(),
                    message: "source_path must remain project-relative".to_string(),
                }
            })?;
        if source_path != metadata.source_path {
            return Err(AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "source_path must use normalized project separators".to_string(),
            });
        }
        if !metadata.source_identity.is_valid()
            || metadata.source_identity != source_identity(&source_path)
        {
            return Err(AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "source identity does not match the normalized source path".to_string(),
            });
        }
        if !metadata.committed_checksum.is_valid() || !metadata.provenance.is_valid() {
            return Err(AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "snapshot provenance is invalid or unsupported".to_string(),
            });
        }
        AutosaveExtension::parse(metadata.extension.as_str()).map_err(|_| {
            AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "snapshot extension is invalid".to_string(),
            }
        })?;
        Ok(metadata)
    }
}

pub(super) fn snapshot_metadata_path(directory: &Path, sequence: u64) -> PathBuf {
    directory.join(format!("{sequence}{SNAPSHOT_METADATA_SUFFIX}"))
}

pub(super) fn snapshot_metadata_sequence(name: &str) -> Option<u64> {
    name.strip_suffix(SNAPSHOT_METADATA_SUFFIX)
        .and_then(|sequence| sequence.parse::<u64>().ok())
        .filter(|sequence| *sequence != 0)
}

fn source_identity(source_path: &AutosaveSourcePath) -> AutosaveContentDigest {
    let source = source_path
        .as_path()
        .to_str()
        .expect("autosave source paths are validated as UTF-8");
    AutosaveContentDigest::from_bytes(source.as_bytes())
}
