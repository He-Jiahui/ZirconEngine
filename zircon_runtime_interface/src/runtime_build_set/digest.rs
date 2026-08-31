use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use super::ZrRuntimeIdentityFormatError;

/// Canonical lowercase hexadecimal digest carried by runtime release metadata.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct ZrRuntimeDigestV1(String);

impl ZrRuntimeDigestV1 {
    pub fn parse(value: impl Into<String>) -> Result<Self, ZrRuntimeIdentityFormatError> {
        let value = value.into();
        let is_lowercase_hex = value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
        if !is_lowercase_hex {
            return Err(ZrRuntimeIdentityFormatError::Digest {
                kind: "runtime digest",
                value,
            });
        }
        Ok(Self(value))
    }

    pub fn sha256(bytes: impl AsRef<[u8]>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes.as_ref());
        Self(format!("{:x}", hasher.finalize()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ZrRuntimeDigestV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
