use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::AutosaveContentDigest;

/// The authoritative source state observed while a document snapshot was captured.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutosaveSourceDigest {
    Missing,
    Present(AutosaveContentDigest),
}

impl AutosaveSourceDigest {
    pub fn observe(path: &Path) -> io::Result<Self> {
        match AutosaveContentDigest::from_file(path) {
            Ok(digest) => Ok(Self::Present(digest)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::Missing),
            Err(error) => Err(error),
        }
    }

    pub const fn missing() -> Self {
        Self::Missing
    }

    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Self::Missing => true,
            Self::Present(digest) => digest.is_valid(),
        }
    }
}
