use std::path::Path;

use serde::{Deserialize, Serialize};

use super::AutosaveSourcePath;
use crate::core::recovery::AutosaveError;

pub(super) const RECOVERY_METADATA_FILE_NAME: &str = "recovery.json";
const RECOVERY_METADATA_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct AutosaveRecoveryMetadata {
    version: u32,
    source_path: AutosaveSourcePath,
}

impl AutosaveRecoveryMetadata {
    pub(super) fn from_source_path(source_path: AutosaveSourcePath) -> Self {
        Self {
            version: RECOVERY_METADATA_VERSION,
            source_path,
        }
    }

    pub(super) fn source_path(&self) -> &AutosaveSourcePath {
        &self.source_path
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
        AutosaveSourcePath::parse(metadata.source_path.as_path()).map_err(|_| {
            AutosaveError::InvalidRecoveryMetadata {
                path: path.to_path_buf(),
                message: "source_path must remain project-relative".to_string(),
            }
        })?;
        Ok(metadata)
    }
}
