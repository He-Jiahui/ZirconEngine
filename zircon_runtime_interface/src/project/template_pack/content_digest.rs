use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

/// Exact SHA-256 identity of a template's source entries before project identity rewriting.
///
/// The digest is computed over a canonical sequence of sorted path and byte-length records, so
/// it remains stable when the embedded table declaration order changes while any source payload
/// or relative path changes the identity.
/// 哈希输入使用版本域标签，并以小端长度前缀分隔每个路径和文件字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectTemplateContentDigest([u8; 32]);

impl ProjectTemplateContentDigest {
    pub(super) fn from_entries<'a, I>(entries: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a [u8])>,
    {
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        entries.sort_unstable_by(|left, right| {
            left.0
                .as_bytes()
                .cmp(right.0.as_bytes())
                .then_with(|| left.1.cmp(right.1))
        });

        let mut hasher = Sha256::new();
        hasher.update(b"zircon-project-template-content-v1\0");
        for (path, bytes) in entries {
            update_length_prefixed(&mut hasher, path.len() as u64);
            hasher.update(path.as_bytes());
            update_length_prefixed(&mut hasher, bytes.len() as u64);
            hasher.update(bytes);
        }
        Self(hasher.finalize().into())
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        self.to_string()
    }

    /// 解析由 64 个小写十六进制字符组成的 SHA-256 持久表示。
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectTemplateContentDigestParseError> {
        let value = value.into();
        if value.len() != 64 {
            return Err(ProjectTemplateContentDigestParseError { value });
        }
        let mut bytes = [0; 32];
        for (target, pair) in bytes.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
            let Some(high) = hex_value(pair[0]) else {
                return Err(ProjectTemplateContentDigestParseError {
                    value: value.clone(),
                });
            };
            let Some(low) = hex_value(pair[1]) else {
                return Err(ProjectTemplateContentDigestParseError {
                    value: value.clone(),
                });
            };
            *target = high << 4 | low;
        }
        Ok(Self(bytes))
    }
}

impl Display for ProjectTemplateContentDigest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Serialize for ProjectTemplateContentDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ProjectTemplateContentDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("project template content digest must be 64 lowercase hexadecimal characters: {value}")]
pub struct ProjectTemplateContentDigestParseError {
    value: String,
}

fn update_length_prefixed(hasher: &mut Sha256, length: u64) {
    hasher.update(length.to_le_bytes());
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/content_digest.rs"]
mod tests;
