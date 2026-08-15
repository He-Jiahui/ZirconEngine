use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactBlobError, IblBakeArtifactCandidate,
    IblBakeArtifactDescriptor, IblBakeArtifactProducer, IblBakeArtifactRequest, IblBakeKey,
    IBL_BAKE_ALGORITHM_VERSION,
};
use crate::core::resource::io::atomic_write;

pub const IBL_BAKE_RUNTIME_CACHE_DIRECTORY: &str = "render/ibl";
pub const IBL_BAKE_RUNTIME_CACHE_EXTENSION: &str = "zribl";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactCacheStore {
    cache_root: PathBuf,
}

impl IblBakeArtifactCacheStore {
    pub fn new(cache_root: impl Into<PathBuf>) -> Self {
        Self {
            cache_root: cache_root.into(),
        }
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    pub fn runtime_cache_path(&self, request: &IblBakeArtifactRequest) -> PathBuf {
        let source_hash = ibl_bake_artifact_request_identity_hash(request);
        self.cache_root
            .join(IBL_BAKE_RUNTIME_CACHE_DIRECTORY)
            .join(format!("v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
            .join(source_hash)
            .join(format!(
                "face_{:04}_mips_{:02}.{}",
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
                IBL_BAKE_RUNTIME_CACHE_EXTENSION
            ))
    }

    pub fn runtime_cache_path_for_descriptor(
        &self,
        descriptor: IblBakeArtifactDescriptor,
    ) -> PathBuf {
        let request = IblBakeArtifactRequest::new(
            descriptor.bake_key(),
            descriptor.source_face_size(),
            descriptor.source_mip_count(),
        )
        .with_pmrem_layout(descriptor.face_size(), descriptor.mip_count())
        .with_required_contents(descriptor.contents());
        self.runtime_cache_path(&request)
    }

    pub fn write_runtime_cache(
        &self,
        blob: &IblBakeArtifactBlob,
    ) -> Result<PathBuf, IblBakeArtifactCacheError> {
        if blob.descriptor().producer() != IblBakeArtifactProducer::RendererGpuRuntime {
            return Err(IblBakeArtifactCacheError::InvalidProducer {
                producer: blob.descriptor().producer(),
            });
        }
        let path = self.runtime_cache_path_for_descriptor(blob.descriptor());
        atomic_write(&path, &blob.encode()).map_err(|source| IblBakeArtifactCacheError::Write {
            path: path.clone(),
            source,
        })?;
        Ok(path)
    }

    pub fn read_runtime_cache(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<IblBakeArtifactCacheRead, IblBakeArtifactCacheError> {
        let path = self.runtime_cache_path(request);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(IblBakeArtifactCacheRead::Missing);
            }
            Err(source) => {
                return Err(IblBakeArtifactCacheError::Read {
                    path: path.clone(),
                    source,
                });
            }
        };

        Ok(
            match IblBakeArtifactBlob::decode_current_runtime_cache_for_request(request, &bytes) {
                Ok(blob) => IblBakeArtifactCacheRead::Hit(blob),
                Err(error) => IblBakeArtifactCacheRead::Rejected(error),
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactCacheRead {
    Hit(IblBakeArtifactBlob),
    Missing,
    Rejected(IblBakeArtifactBlobError),
}

impl IblBakeArtifactCacheRead {
    pub fn candidate(&self) -> Option<IblBakeArtifactCandidate> {
        match self {
            Self::Hit(blob) => Some(IblBakeArtifactCandidate::runtime_cache(blob.descriptor())),
            Self::Missing | Self::Rejected(_) => None,
        }
    }

    pub const fn is_hit(&self) -> bool {
        matches!(self, Self::Hit(_))
    }
}

#[derive(Debug, Error)]
pub enum IblBakeArtifactCacheError {
    #[error("runtime IBL cache requires a renderer GPU artifact, got {producer:?}")]
    InvalidProducer { producer: IblBakeArtifactProducer },
    #[error("create IBL bake artifact cache directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("write IBL bake artifact cache {path:?}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("read IBL bake artifact cache {path:?}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub(super) fn ibl_bake_artifact_request_identity_hash(request: &IblBakeArtifactRequest) -> String {
    let mut hasher = blake3::Hasher::new();
    update_bake_key_hash(&mut hasher, request.bake_key());
    hasher.update(&request.source_face_size().to_le_bytes());
    hasher.update(&request.source_mip_count().to_le_bytes());
    hasher.update(&request.pmrem_face_size().to_le_bytes());
    hasher.update(&request.pmrem_mip_count().to_le_bytes());
    hasher.update(&request.required_contents().bits().to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

fn update_bake_key_hash(hasher: &mut blake3::Hasher, bake_key: IblBakeKey) {
    hasher.update(&bake_key.source_kind.to_le_bytes());
    hasher.update(&bake_key.source_revision.to_le_bytes());
    update_u32_array_hash(hasher, &bake_key.horizon_color);
    update_u32_array_hash(hasher, &bake_key.zenith_color);
    update_u32_array_hash(hasher, &bake_key.ground_color);
    update_u32_array_hash(hasher, &bake_key.source_hash);
}

fn update_u32_array_hash(hasher: &mut blake3::Hasher, values: &[u32; 4]) {
    for value in values {
        hasher.update(&value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_cache_writer_uses_runtime_atomic_publication() {
        let source = include_str!("ibl_bake_artifact_cache.rs");
        let writer = source
            .split("pub fn write_runtime_cache(")
            .nth(1)
            .and_then(|writer| writer.split("pub fn read_runtime_cache(").next())
            .expect("runtime cache store must retain its writer");

        assert!(source.contains("core::resource::io::atomic_write"));
        assert!(writer.contains("atomic_write("));
        assert!(!writer.contains("fs::write("));
    }
}
