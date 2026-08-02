use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::core::framework::render::{
    IBL_BAKE_ALGORITHM_VERSION, IblBakeArtifactBlob, IblBakeArtifactBlobCandidate,
    IblBakeArtifactBlobError, IblBakeArtifactDescriptor, IblBakeArtifactPayload,
    IblBakeArtifactPayloadError, IblBakeArtifactRequest, SourceCubemapIrradianceCube,
    SourceCubemapMipChain,
};

use super::ibl_bake_artifact_cache::ibl_bake_artifact_request_identity_hash;

pub const IBL_BAKE_ASSET_DERIVED_DIRECTORY: &str = "render/ibl-derived";
pub const IBL_BAKE_ASSET_DERIVED_EXTENSION: &str = "zribl";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactAssetDerivedStore {
    cache_root: PathBuf,
}

impl IblBakeArtifactAssetDerivedStore {
    pub fn new(cache_root: impl Into<PathBuf>) -> Self {
        Self {
            cache_root: cache_root.into(),
        }
    }

    pub fn asset_derived_path(&self, request: &IblBakeArtifactRequest) -> PathBuf {
        let source_hash = ibl_bake_artifact_request_identity_hash(request);
        self.cache_root
            .join(IBL_BAKE_ASSET_DERIVED_DIRECTORY)
            .join(format!("v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
            .join(source_hash)
            .join(format!(
                "face_{:04}_mips_{:02}.{}",
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
                IBL_BAKE_ASSET_DERIVED_EXTENSION
            ))
    }

    pub fn asset_derived_path_for_descriptor(
        &self,
        descriptor: IblBakeArtifactDescriptor,
    ) -> PathBuf {
        self.asset_derived_path(&request_for_descriptor(descriptor))
    }

    pub fn write_asset_derived_blob(
        &self,
        blob: &IblBakeArtifactBlob,
    ) -> Result<IblBakeArtifactAssetDerivedWriteReport, IblBakeArtifactAssetDerivedError> {
        let path = self.asset_derived_path_for_descriptor(blob.descriptor());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                IblBakeArtifactAssetDerivedError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
        fs::write(&path, blob.encode()).map_err(|source| {
            IblBakeArtifactAssetDerivedError::Write {
                path: path.clone(),
                source,
            }
        })?;
        Ok(IblBakeArtifactAssetDerivedWriteReport {
            path,
            descriptor: blob.descriptor(),
            encoded_len: blob.encoded_len(),
            payload_len: blob.payload().bytes().len(),
        })
    }

    pub fn write_source_cubemap_asset_derived_artifact(
        &self,
        request: &IblBakeArtifactRequest,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<IblBakeArtifactAssetDerivedWriteReport, IblBakeArtifactAssetDerivedError> {
        let descriptor = IblBakeArtifactDescriptor::current_for_request(request);
        let payload =
            IblBakeArtifactPayload::from_source_cubemap(descriptor, cubemap, irradiance_cube)
                .map_err(|error| IblBakeArtifactAssetDerivedError::Payload { error })?;
        self.write_asset_derived_blob(&IblBakeArtifactBlob::from_payload(payload))
    }

    pub fn read_asset_derived_artifact(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<IblBakeArtifactAssetDerivedRead, IblBakeArtifactAssetDerivedError> {
        let path = self.asset_derived_path(request);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(IblBakeArtifactAssetDerivedRead::Missing);
            }
            Err(source) => {
                return Err(IblBakeArtifactAssetDerivedError::Read { path, source });
            }
        };
        Ok(
            match IblBakeArtifactBlob::decode_current_for_request(request, &bytes) {
                Ok(blob) => IblBakeArtifactAssetDerivedRead::Hit(blob),
                Err(error) => IblBakeArtifactAssetDerivedRead::Rejected(error),
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactAssetDerivedRead {
    Hit(IblBakeArtifactBlob),
    Missing,
    Rejected(IblBakeArtifactBlobError),
}

impl IblBakeArtifactAssetDerivedRead {
    pub fn blob(&self) -> Option<&IblBakeArtifactBlob> {
        match self {
            Self::Hit(blob) => Some(blob),
            Self::Missing | Self::Rejected(_) => None,
        }
    }

    pub fn candidate(&self) -> Option<IblBakeArtifactBlobCandidate> {
        self.blob()
            .cloned()
            .map(IblBakeArtifactBlobCandidate::asset_derived)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactAssetDerivedWriteReport {
    path: PathBuf,
    descriptor: IblBakeArtifactDescriptor,
    encoded_len: usize,
    payload_len: usize,
}

impl IblBakeArtifactAssetDerivedWriteReport {
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }

    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }

    pub const fn payload_len(&self) -> usize {
        self.payload_len
    }
}

#[derive(Debug, Error)]
pub enum IblBakeArtifactAssetDerivedError {
    #[error("create IBL bake asset-derived artifact directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("write IBL bake asset-derived artifact {path:?}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("read IBL bake asset-derived artifact {path:?}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("build IBL bake asset-derived artifact payload: {error:?}")]
    Payload { error: IblBakeArtifactPayloadError },
}

fn request_for_descriptor(descriptor: IblBakeArtifactDescriptor) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        descriptor.bake_key(),
        descriptor.source_face_size(),
        descriptor.source_mip_count(),
    )
    .with_pmrem_layout(descriptor.face_size(), descriptor.mip_count())
    .with_required_contents(descriptor.contents())
}
