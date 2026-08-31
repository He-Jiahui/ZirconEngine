use serde::{Deserialize, Deserializer, Serialize};

use super::{ZrRuntimeDigestV1, ZrRuntimeIdentityFormatError};

/// Identifies one lockstep internal product build across Host, Runtime, and Editor.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct ZrRuntimeBuildSetId(String);

impl ZrRuntimeBuildSetId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ZrRuntimeIdentityFormatError> {
        let value = value.into();
        let is_lowercase_hex = value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
        if !is_lowercase_hex {
            return Err(ZrRuntimeIdentityFormatError::Digest {
                kind: "runtime BuildSet id",
                value,
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_sha256_digest(digest: ZrRuntimeDigestV1) -> Self {
        Self(digest.as_str().to_owned())
    }
}

impl<'de> Deserialize<'de> for ZrRuntimeBuildSetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
