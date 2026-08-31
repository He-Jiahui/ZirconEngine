use std::path::PathBuf;

use thiserror::Error;

use crate::asset::{AssetUri, AssetUuid};

#[derive(Debug, Error)]
pub enum AssetRegistryError {
    #[error("asset registry I/O failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to encode asset registry persistence at {path}: {source}")]
    EncodePersistence {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to decode asset registry persistence at {path}: {source}")]
    DecodePersistence {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "asset registry persistence at {path} has version {found}; supported version is {supported}"
    )]
    UnsupportedPersistenceVersion {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    #[error("asset uuid {uuid} is not registered")]
    AssetUuidNotFound { uuid: AssetUuid },
    #[error("asset path {path} is not registered")]
    AssetPathNotFound { path: AssetUri },
    #[error("asset reference {uuid} with path hint {path} is dangling")]
    AssetReferenceNotFound { uuid: AssetUuid, path: AssetUri },
    #[error("asset id {id} is not registered")]
    AssetIdNotFound { id: crate::asset::AssetId },
    #[error("asset registry metadata scan rejected link or reparse point {path} below {root}")]
    UnsafeMetadataLink { root: PathBuf, path: PathBuf },
    #[error("asset registry metadata path {path} escapes canonical root {root}")]
    MetadataPathEscapesRoot { root: PathBuf, path: PathBuf },
    #[error("asset registry metadata scan detected a directory cycle at {path} below {root}")]
    MetadataDirectoryCycle { root: PathBuf, path: PathBuf },
    #[error("asset registry contains duplicate uuid {uuid} for {first} and {second}")]
    DuplicateUuid {
        uuid: AssetUuid,
        first: AssetUri,
        second: AssetUri,
    },
    #[error("asset registry contains duplicate path {path} for {first} and {second}")]
    DuplicatePath {
        path: AssetUri,
        first: AssetUuid,
        second: AssetUuid,
    },
    #[error(
        "asset source relocation from {source_uri} to {target} does not preserve the registered entry set: {reason}"
    )]
    SourceRelocationIdentityMismatch {
        source_uri: AssetUri,
        target: AssetUri,
        reason: String,
    },
}

impl PartialEq for AssetRegistryError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Io {
                    path: left,
                    source: left_source,
                },
                Self::Io {
                    path: right,
                    source: right_source,
                },
            ) => {
                left == right
                    && left_source.kind() == right_source.kind()
                    && left_source.to_string() == right_source.to_string()
            }
            (
                Self::EncodePersistence {
                    path: left,
                    source: left_source,
                },
                Self::EncodePersistence {
                    path: right,
                    source: right_source,
                },
            )
            | (
                Self::DecodePersistence {
                    path: left,
                    source: left_source,
                },
                Self::DecodePersistence {
                    path: right,
                    source: right_source,
                },
            ) => {
                left == right
                    && left_source.classify() == right_source.classify()
                    && left_source.to_string() == right_source.to_string()
            }
            (
                Self::UnsupportedPersistenceVersion {
                    path: left_path,
                    found: left_found,
                    supported: left_supported,
                },
                Self::UnsupportedPersistenceVersion {
                    path: right_path,
                    found: right_found,
                    supported: right_supported,
                },
            ) => {
                left_path == right_path
                    && left_found == right_found
                    && left_supported == right_supported
            }
            (Self::AssetUuidNotFound { uuid: left }, Self::AssetUuidNotFound { uuid: right }) => {
                left == right
            }
            (Self::AssetPathNotFound { path: left }, Self::AssetPathNotFound { path: right }) => {
                left == right
            }
            (
                Self::AssetReferenceNotFound {
                    uuid: left_uuid,
                    path: left_path,
                },
                Self::AssetReferenceNotFound {
                    uuid: right_uuid,
                    path: right_path,
                },
            ) => left_uuid == right_uuid && left_path == right_path,
            (Self::AssetIdNotFound { id: left }, Self::AssetIdNotFound { id: right }) => {
                left == right
            }
            (
                Self::UnsafeMetadataLink {
                    root: left_root,
                    path: left_path,
                },
                Self::UnsafeMetadataLink {
                    root: right_root,
                    path: right_path,
                },
            )
            | (
                Self::MetadataPathEscapesRoot {
                    root: left_root,
                    path: left_path,
                },
                Self::MetadataPathEscapesRoot {
                    root: right_root,
                    path: right_path,
                },
            )
            | (
                Self::MetadataDirectoryCycle {
                    root: left_root,
                    path: left_path,
                },
                Self::MetadataDirectoryCycle {
                    root: right_root,
                    path: right_path,
                },
            ) => left_root == right_root && left_path == right_path,
            (
                Self::DuplicateUuid {
                    uuid: left_uuid,
                    first: left_first,
                    second: left_second,
                },
                Self::DuplicateUuid {
                    uuid: right_uuid,
                    first: right_first,
                    second: right_second,
                },
            ) => {
                left_uuid == right_uuid && left_first == right_first && left_second == right_second
            }
            (
                Self::DuplicatePath {
                    path: left_path,
                    first: left_first,
                    second: left_second,
                },
                Self::DuplicatePath {
                    path: right_path,
                    first: right_first,
                    second: right_second,
                },
            ) => {
                left_path == right_path && left_first == right_first && left_second == right_second
            }
            (
                Self::SourceRelocationIdentityMismatch {
                    source_uri: left_source,
                    target: left_target,
                    reason: left_reason,
                },
                Self::SourceRelocationIdentityMismatch {
                    source_uri: right_source,
                    target: right_target,
                    reason: right_reason,
                },
            ) => {
                left_source == right_source
                    && left_target == right_target
                    && left_reason == right_reason
            }
            _ => false,
        }
    }
}

impl AssetRegistryError {
    pub(super) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
