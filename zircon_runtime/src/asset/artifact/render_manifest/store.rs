use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bincode::Options;
use thiserror::Error;

use crate::core::resource::UntypedResourceHandle;
use crate::core::resource::io::atomic_write_new;

use super::{
    RenderArtifactBlockDescriptor, RenderArtifactContentId, RenderArtifactManifest,
    RenderArtifactManifestError,
};

const RENDER_ARTIFACT_STORE_DIRECTORY: &str = "render-artifacts";
const RENDER_ARTIFACT_BLOCK_DIRECTORY: &str = "blocks";
const RENDER_ARTIFACT_MANIFEST_DIRECTORY: &str = "manifests";
const RENDER_ARTIFACT_BLOCK_EXTENSION: &str = "zr-render-block";
const RENDER_ARTIFACT_MANIFEST_EXTENSION: &str = "zr-render-manifest";
const RENDER_ARTIFACT_MANIFEST_MAGIC: &[u8] = b"ZRRMAN01";

mod cook_publication;

pub use cook_publication::{
    RenderArtifactCookPublicationError, RenderArtifactCookPublicationReport,
    publish_render_artifact_cook_output,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactStoreLimits {
    max_manifest_bytes: u64,
    max_encoded_block_bytes: u64,
}

impl RenderArtifactStoreLimits {
    pub const fn new(max_manifest_bytes: u64, max_encoded_block_bytes: u64) -> Self {
        Self {
            max_manifest_bytes,
            max_encoded_block_bytes,
        }
    }

    pub const fn max_manifest_bytes(self) -> u64 {
        self.max_manifest_bytes
    }

    pub const fn max_encoded_block_bytes(self) -> u64 {
        self.max_encoded_block_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactPublishStatus {
    Published,
    Reused,
}

#[derive(Debug, Error)]
pub enum RenderArtifactStoreError {
    #[error("render artifact {byte_kind} byte limit must be non-zero")]
    ZeroByteLimit { byte_kind: &'static str },
    #[error("render artifact {byte_kind} has {actual} bytes, exceeding limit {limit}")]
    ByteLimitExceeded {
        byte_kind: &'static str,
        actual: u64,
        limit: u64,
    },
    #[error("render artifact {byte_kind} byte count {actual} does not fit this address space")]
    AddressSpaceOverflow {
        byte_kind: &'static str,
        actual: u64,
    },
    #[error(
        "render artifact block size mismatch: descriptor records {expected}, payload has {actual}"
    )]
    BlockSizeMismatch { expected: u64, actual: u64 },
    #[error("render artifact block content does not match content id {content_id:?}")]
    BlockContentHashMismatch { content_id: RenderArtifactContentId },
    #[error("render artifact manifest references unpublished block {content_id:?}")]
    MissingPublishedBlock { content_id: RenderArtifactContentId },
    #[error("render artifact manifest does not begin with the supported magic")]
    ManifestMagicMismatch,
    #[error("render artifact manifest identity does not match its requested key")]
    ManifestIdentityMismatch,
    #[error("a different render artifact manifest already owns this resource revision and target")]
    ManifestConflict,
    #[error("render artifact manifest serialization failed")]
    ManifestSerialize(#[source] Box<bincode::ErrorKind>),
    #[error("render artifact manifest deserialization failed")]
    ManifestDeserialize(#[source] Box<bincode::ErrorKind>),
    #[error(transparent)]
    ManifestValidation(#[from] RenderArtifactManifestError),
    #[error("render artifact store I/O failed")]
    Io(#[from] io::Error),
}

#[derive(Clone, Debug)]
pub struct RenderArtifactStore {
    root: Arc<PathBuf>,
}

impl RenderArtifactStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: Arc::new(root),
        }
    }

    pub fn publish_block(
        &self,
        descriptor: &RenderArtifactBlockDescriptor,
        encoded_bytes: &[u8],
        limits: RenderArtifactStoreLimits,
    ) -> Result<RenderArtifactPublishStatus, RenderArtifactStoreError> {
        validate_limit(
            "encoded block",
            descriptor.encoded_bytes(),
            limits.max_encoded_block_bytes(),
        )?;
        let actual_bytes = usize_bytes(encoded_bytes.len(), "encoded block")?;
        if actual_bytes != descriptor.encoded_bytes() {
            return Err(RenderArtifactStoreError::BlockSizeMismatch {
                expected: descriptor.encoded_bytes(),
                actual: actual_bytes,
            });
        }
        if content_id_for(encoded_bytes) != descriptor.content_id() {
            return Err(RenderArtifactStoreError::BlockContentHashMismatch {
                content_id: descriptor.content_id(),
            });
        }

        let path = self.block_path(descriptor.content_id());
        match atomic_write_new(&path, encoded_bytes) {
            Ok(()) => Ok(RenderArtifactPublishStatus::Published),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                self.read_block(descriptor, limits)?;
                Ok(RenderArtifactPublishStatus::Reused)
            }
            Err(error) => Err(RenderArtifactStoreError::Io(error)),
        }
    }

    pub fn read_block(
        &self,
        descriptor: &RenderArtifactBlockDescriptor,
        limits: RenderArtifactStoreLimits,
    ) -> Result<Arc<[u8]>, RenderArtifactStoreError> {
        validate_limit(
            "encoded block",
            descriptor.encoded_bytes(),
            limits.max_encoded_block_bytes(),
        )?;
        let path = self.block_path(descriptor.content_id());
        let bytes = read_bounded_file(&path, limits.max_encoded_block_bytes(), "encoded block")?;
        let actual_bytes = usize_bytes(bytes.len(), "encoded block")?;
        if actual_bytes != descriptor.encoded_bytes() {
            return Err(RenderArtifactStoreError::BlockSizeMismatch {
                expected: descriptor.encoded_bytes(),
                actual: actual_bytes,
            });
        }
        if content_id_for(&bytes) != descriptor.content_id() {
            return Err(RenderArtifactStoreError::BlockContentHashMismatch {
                content_id: descriptor.content_id(),
            });
        }
        Ok(bytes.into())
    }

    pub fn block_exists(&self, content_id: RenderArtifactContentId) -> bool {
        self.block_path(content_id).is_file()
    }

    pub fn publish_manifest(
        &self,
        manifest: &RenderArtifactManifest,
        limits: RenderArtifactStoreLimits,
    ) -> Result<RenderArtifactPublishStatus, RenderArtifactStoreError> {
        manifest.validate()?;
        validate_limit("manifest", 1, limits.max_manifest_bytes())?;
        for block in manifest.blocks() {
            match self.read_block(block, limits) {
                Ok(_) => {}
                Err(RenderArtifactStoreError::Io(error))
                    if error.kind() == io::ErrorKind::NotFound =>
                {
                    return Err(RenderArtifactStoreError::MissingPublishedBlock {
                        content_id: block.content_id(),
                    });
                }
                Err(error) => return Err(error),
            }
        }
        self.publish_verified_manifest(manifest, limits)
    }

    fn publish_verified_manifest(
        &self,
        manifest: &RenderArtifactManifest,
        limits: RenderArtifactStoreLimits,
    ) -> Result<RenderArtifactPublishStatus, RenderArtifactStoreError> {
        manifest.validate()?;
        let payload = serialize_manifest(manifest, limits)?;
        let path = self.manifest_path(
            manifest.resource(),
            manifest.asset_revision(),
            manifest.target_platform(),
        );
        match atomic_write_new(&path, &payload) {
            Ok(()) => Ok(RenderArtifactPublishStatus::Published),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let current = self.read_manifest(
                    manifest.resource(),
                    manifest.asset_revision(),
                    manifest.target_platform(),
                    limits,
                )?;
                if current == *manifest {
                    Ok(RenderArtifactPublishStatus::Reused)
                } else {
                    Err(RenderArtifactStoreError::ManifestConflict)
                }
            }
            Err(error) => Err(RenderArtifactStoreError::Io(error)),
        }
    }

    pub fn read_manifest(
        &self,
        resource: UntypedResourceHandle,
        asset_revision: u64,
        target_platform: &str,
        limits: RenderArtifactStoreLimits,
    ) -> Result<RenderArtifactManifest, RenderArtifactStoreError> {
        let path = self.manifest_path(resource, asset_revision, target_platform);
        let payload = read_bounded_file(&path, limits.max_manifest_bytes(), "manifest")?;
        let Some(bytes) = payload.strip_prefix(RENDER_ARTIFACT_MANIFEST_MAGIC) else {
            return Err(RenderArtifactStoreError::ManifestMagicMismatch);
        };
        let manifest: RenderArtifactManifest = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(limits.max_manifest_bytes())
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(RenderArtifactStoreError::ManifestDeserialize)?;
        manifest.validate()?;
        if manifest.resource() != resource
            || manifest.asset_revision() != asset_revision
            || manifest.target_platform() != target_platform
        {
            return Err(RenderArtifactStoreError::ManifestIdentityMismatch);
        }
        Ok(manifest)
    }

    fn block_path(&self, content_id: RenderArtifactContentId) -> PathBuf {
        let hash = content_id_hex(content_id);
        self.root
            .join(RENDER_ARTIFACT_STORE_DIRECTORY)
            .join(RENDER_ARTIFACT_BLOCK_DIRECTORY)
            .join(&hash[..2])
            .join(format!("{hash}.{RENDER_ARTIFACT_BLOCK_EXTENSION}"))
    }

    fn manifest_path(
        &self,
        resource: UntypedResourceHandle,
        asset_revision: u64,
        target_platform: &str,
    ) -> PathBuf {
        let target_hash = blake3::hash(target_platform.as_bytes()).to_hex();
        self.root
            .join(RENDER_ARTIFACT_STORE_DIRECTORY)
            .join(RENDER_ARTIFACT_MANIFEST_DIRECTORY)
            .join(format!(
                "{:02}",
                super::validation::resource_kind_tag(resource.kind())
            ))
            .join(resource.id().to_string())
            .join(asset_revision.to_string())
            .join(format!(
                "{target_hash}.{RENDER_ARTIFACT_MANIFEST_EXTENSION}"
            ))
    }
}

fn serialize_manifest(
    manifest: &RenderArtifactManifest,
    limits: RenderArtifactStoreLimits,
) -> Result<Vec<u8>, RenderArtifactStoreError> {
    validate_limit("manifest", 1, limits.max_manifest_bytes())?;
    let encoded = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(limits.max_manifest_bytes())
        .serialize(manifest)
        .map_err(RenderArtifactStoreError::ManifestSerialize)?;
    let Some(capacity) = RENDER_ARTIFACT_MANIFEST_MAGIC
        .len()
        .checked_add(encoded.len())
    else {
        return Err(RenderArtifactStoreError::AddressSpaceOverflow {
            byte_kind: "manifest",
            actual: u64::MAX,
        });
    };
    let total_bytes = usize_bytes(capacity, "manifest")?;
    validate_limit("manifest", total_bytes, limits.max_manifest_bytes())?;
    let mut payload = Vec::with_capacity(capacity);
    payload.extend_from_slice(RENDER_ARTIFACT_MANIFEST_MAGIC);
    payload.extend_from_slice(&encoded);
    Ok(payload)
}

fn read_bounded_file(
    path: &Path,
    limit: u64,
    byte_kind: &'static str,
) -> Result<Vec<u8>, RenderArtifactStoreError> {
    validate_limit(byte_kind, 1, limit)?;
    let file = File::open(path)?;
    let metadata_bytes = file.metadata()?.len();
    validate_limit(byte_kind, metadata_bytes, limit)?;
    let initial_capacity = usize::try_from(metadata_bytes).map_err(|_| {
        RenderArtifactStoreError::AddressSpaceOverflow {
            byte_kind,
            actual: metadata_bytes,
        }
    })?;
    let mut bytes = Vec::with_capacity(initial_capacity);
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    let actual_bytes = usize_bytes(bytes.len(), byte_kind)?;
    validate_limit(byte_kind, actual_bytes, limit)?;
    Ok(bytes)
}

fn validate_limit(
    byte_kind: &'static str,
    actual: u64,
    limit: u64,
) -> Result<(), RenderArtifactStoreError> {
    if limit == 0 {
        return Err(RenderArtifactStoreError::ZeroByteLimit { byte_kind });
    }
    if actual > limit {
        return Err(RenderArtifactStoreError::ByteLimitExceeded {
            byte_kind,
            actual,
            limit,
        });
    }
    Ok(())
}

fn content_id_for(bytes: &[u8]) -> RenderArtifactContentId {
    RenderArtifactContentId::from_bytes(*blake3::hash(bytes).as_bytes())
}

fn usize_bytes(bytes: usize, byte_kind: &'static str) -> Result<u64, RenderArtifactStoreError> {
    u64::try_from(bytes).map_err(|_| RenderArtifactStoreError::AddressSpaceOverflow {
        byte_kind,
        actual: u64::MAX,
    })
}

fn content_id_hex(content_id: RenderArtifactContentId) -> String {
    blake3::Hash::from_bytes(*content_id.as_bytes())
        .to_hex()
        .to_string()
}

#[cfg(test)]
#[path = "store/tests.rs"]
mod tests;
