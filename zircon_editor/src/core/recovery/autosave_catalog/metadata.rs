use std::path::Path;

use serde::{Deserialize, Serialize};

use super::AutosaveSourcePath;
use crate::core::recovery::{AutosaveContentDigest, AutosaveError};

pub(super) const RECOVERY_METADATA_FILE_NAME: &str = "recovery.json";
const RECOVERY_METADATA_VERSION: u32 = 2;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct AutosaveRecoveryMetadata {
    version: u32,
    source_path: AutosaveSourcePath,
    source_identity: AutosaveContentDigest,
}

impl AutosaveRecoveryMetadata {
    pub(super) fn from_source_path(source_path: AutosaveSourcePath) -> Self {
        Self {
            version: RECOVERY_METADATA_VERSION,
            source_identity: source_identity(&source_path),
            source_path,
        }
    }

    pub(super) fn source_path(&self) -> &AutosaveSourcePath {
        &self.source_path
    }

    pub(super) fn source_identity(&self) -> &AutosaveContentDigest {
        &self.source_identity
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
        if metadata.version != RECOVERY_METADATA_VERSION {
            return Err(AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "unsupported version".to_string(),
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
            || metadata.source_identity != source_identity(&metadata.source_path)
        {
            return Err(AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "source identity does not match the normalized source path".to_string(),
            });
        }
        Ok(metadata)
    }
}

fn source_identity(source_path: &AutosaveSourcePath) -> AutosaveContentDigest {
    let source = source_path
        .as_path()
        .to_str()
        .expect("autosave source paths are validated as UTF-8");
    AutosaveContentDigest::from_bytes(source.as_bytes())
}
