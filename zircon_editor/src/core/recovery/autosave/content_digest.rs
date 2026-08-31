use std::fmt;
use std::io::{self, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};

const CONTENT_DIGEST_HEX_LENGTH: usize = 64;
const CONTENT_DIGEST_READ_BUFFER_BYTES: usize = 64 * 1024;

/// A stable BLAKE3 content identity used only for persisted recovery evidence.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AutosaveContentDigest(String);

impl AutosaveContentDigest {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(blake3::hash(bytes).to_hex().to_string())
    }

    pub fn from_file(path: &Path) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::from_reader(file)
    }

    pub fn from_reader(mut reader: impl Read) -> io::Result<Self> {
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0_u8; CONTENT_DIGEST_READ_BUFFER_BYTES];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                return Ok(Self(hasher.finalize().to_hex().to_string()));
            }
            hasher.update(&buffer[..bytes_read]);
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.0.len() == CONTENT_DIGEST_HEX_LENGTH
            && self
                .0
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    }
}

impl fmt::Debug for AutosaveContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("AutosaveContentDigest")
            .field(&self.0)
            .finish()
    }
}
