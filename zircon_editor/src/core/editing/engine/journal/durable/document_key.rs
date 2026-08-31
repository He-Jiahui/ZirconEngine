use std::path::{Path, PathBuf};

/// Stable project-document identity used by journal storage and recovery selection.
///
/// `DocumentId` is a session-local counter, so it must never name a durable journal directory.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JournalDocumentKey {
    value: String,
    source_path: PathBuf,
}

impl JournalDocumentKey {
    pub fn from_project_relative_path(source_path: &Path) -> Result<Self, JournalDocumentKeyError> {
        let Some(source) = source_path.to_str() else {
            return Err(JournalDocumentKeyError::NonUtf8SourcePath {
                path: source_path.to_path_buf(),
            });
        };
        let normalized = normalize_project_relative_path(source).ok_or_else(|| {
            JournalDocumentKeyError::InvalidSourcePath {
                path: source_path.to_path_buf(),
            }
        })?;
        let source_path = normalized.join("/");
        let value = format!("document_{}", blake3::hash(source_path.as_bytes()).to_hex());
        Ok(Self {
            value,
            source_path: PathBuf::from(source_path),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
}

fn normalize_project_relative_path(source: &str) -> Option<Vec<&str>> {
    let bytes = source.as_bytes();
    let has_drive_prefix = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if source.is_empty() || source.starts_with(['/', '\\']) || has_drive_prefix {
        return None;
    }

    let components = source
        .split(['/', '\\'])
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| matches!(*component, "." | ".."))
    {
        return None;
    }
    Some(components)
}

#[derive(Debug, thiserror::Error)]
pub enum JournalDocumentKeyError {
    #[error("journal source path must be a non-empty project-relative path: {path}")]
    InvalidSourcePath { path: PathBuf },
    #[error("journal source path must be UTF-8: {path}")]
    NonUtf8SourcePath { path: PathBuf },
}
