use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::AutosaveError;
use crate::core::recovery::autosave_catalog::AutosaveSourcePath;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveDocumentId(String);

impl AutosaveDocumentId {
    pub fn parse(value: impl Into<String>) -> Result<Self, AutosaveError> {
        let value = value.into();
        if value.is_empty()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_uppercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'-')
            })
        {
            return Err(AutosaveError::InvalidDocumentId { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Derives the persistent recovery identity from the normalized project-relative source.
    pub fn from_source_path(source_path: &AutosaveSourcePath) -> Self {
        let source = source_path
            .as_path()
            .to_str()
            .expect("autosave source paths are validated as UTF-8");
        Self(format!(
            "document_{}",
            blake3::hash(source.as_bytes()).to_hex()
        ))
    }
}

impl Ord for AutosaveDocumentId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for AutosaveDocumentId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveExtension(String);

impl AutosaveExtension {
    pub fn parse(value: impl Into<String>) -> Result<Self, AutosaveError> {
        let value = value.into();
        if value.is_empty()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_uppercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'-')
            })
        {
            return Err(AutosaveError::InvalidExtension { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
