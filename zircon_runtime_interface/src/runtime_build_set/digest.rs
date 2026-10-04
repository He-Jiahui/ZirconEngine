use std::io::{self, Read};

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use super::ZrRuntimeIdentityFormatError;

const STREAM_HASH_BUFFER_BYTES: usize = 64 * 1024;

/// Canonical lowercase hexadecimal digest carried by runtime release metadata.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct ZrRuntimeDigestV1(String);

impl ZrRuntimeDigestV1 {
    /// 构建集清单、产物摘要和组合回执共用规范 SHA-256 文本；这里只校验 64 位小写十六进制，不代替对输入字节求哈希。
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

    /// Hashes a potentially large identity source without retaining it in memory.
    pub fn sha256_reader(mut reader: impl Read) -> io::Result<Self> {
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; STREAM_HASH_BUFFER_BYTES];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(Self(format!("{:x}", hasher.finalize())))
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

#[cfg(test)]
#[path = "tests/digest.rs"]
mod tests;
