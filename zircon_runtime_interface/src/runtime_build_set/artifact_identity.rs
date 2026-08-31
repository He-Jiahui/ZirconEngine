use serde::{Deserialize, Serialize};

use super::{ZrRuntimeDigestV1, ZrRuntimeIdentityFormatError};

/// Names and hashes the exact dynamic library selected by a release manifest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeArtifactIdentityV1 {
    pub file_name: String,
    pub sha256: ZrRuntimeDigestV1,
}

impl ZrRuntimeArtifactIdentityV1 {
    pub fn new(
        file_name: impl Into<String>,
        sha256: ZrRuntimeDigestV1,
    ) -> Result<Self, ZrRuntimeIdentityFormatError> {
        let identity = Self {
            file_name: file_name.into(),
            sha256,
        };
        identity.validate()?;
        Ok(identity)
    }

    pub fn validate(&self) -> Result<(), ZrRuntimeIdentityFormatError> {
        let name = self.file_name.as_str();
        if name.is_empty() || matches!(name, "." | "..") || name.contains(['/', '\\']) {
            return Err(ZrRuntimeIdentityFormatError::ArtifactFileName {
                value: self.file_name.clone(),
            });
        }
        Ok(())
    }
}
