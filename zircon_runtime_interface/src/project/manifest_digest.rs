use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ProjectManifestDigestParseError;

/// Exact BLAKE3 content identity of the manifest bytes accepted during preflight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectManifestDigest([u8; 32]);

impl ProjectManifestDigest {
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        Self(*blake3::hash(bytes.as_ref()).as_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.to_string()
    }

    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectManifestDigestParseError> {
        let value = value.into();
        let expected_hex_characters = blake3::OUT_LEN * 2;
        let source = value.as_bytes();
        if source.len() != expected_hex_characters {
            return Err(ProjectManifestDigestParseError::new(
                value,
                expected_hex_characters,
            ));
        }

        let mut bytes = [0; blake3::OUT_LEN];
        for (target, pair) in bytes.iter_mut().zip(source.chunks_exact(2)) {
            let Some(high) = lowercase_hex_value(pair[0]) else {
                return Err(ProjectManifestDigestParseError::new(
                    value.clone(),
                    expected_hex_characters,
                ));
            };
            let Some(low) = lowercase_hex_value(pair[1]) else {
                return Err(ProjectManifestDigestParseError::new(
                    value.clone(),
                    expected_hex_characters,
                ));
            };
            *target = high << 4 | low;
        }
        Ok(Self(bytes))
    }
}

impl Display for ProjectManifestDigest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Serialize for ProjectManifestDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ProjectManifestDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

fn lowercase_hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
