use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::asset::AssetUri;
use crate::asset::assets::{
    TexturePayload, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE, ZcubeSourceCubemap, ZcubeSourceCubemapError,
    decode_zcube_source_cubemap_bytes, texture_asset_from_source_cubemap_zcube,
};
use crate::core::framework::render::{
    IBL_BAKE_ALGORITHM_VERSION, IblBakeArtifactRequest, SourceCubemapBakeArtifactError,
    SourceCubemapEnvironment, SourceCubemapIrradianceCube, SourceCubemapMipChain,
    build_source_cubemap_from_source_mips, source_cubemap_environment_with_bake_artifact,
};

use super::ibl_bake_artifact_asset_derived::{
    IblBakeArtifactAssetDerivedError, IblBakeArtifactAssetDerivedStore,
    IblBakeArtifactAssetDerivedWriteReport,
};
use super::ibl_bake_artifact_cache::ibl_bake_artifact_request_identity_hash;

pub const IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY: &str = "render/ibl-source";
pub const IBL_SOURCE_CUBEMAP_STAGING_EXTENSION: &str = "zcube";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblSourceCubemapStagingStore {
    cache_root: PathBuf,
}

impl IblSourceCubemapStagingStore {
    pub fn new(cache_root: impl Into<PathBuf>) -> Self {
        Self {
            cache_root: cache_root.into(),
        }
    }

    pub fn source_cubemap_path(&self, request: &IblBakeArtifactRequest) -> PathBuf {
        let source_hash = ibl_bake_artifact_request_identity_hash(request);
        self.cache_root
            .join(IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY)
            .join(format!("v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
            .join(source_hash)
            .join(format!("source.{IBL_SOURCE_CUBEMAP_STAGING_EXTENSION}"))
    }

    pub fn asset_derived_store(&self) -> IblBakeArtifactAssetDerivedStore {
        IblBakeArtifactAssetDerivedStore::new(self.cache_root.clone())
    }

    pub fn write_source_cubemap_zcube(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
    ) -> Result<IblSourceCubemapZcubeWriteReport, IblSourceCubemapStagingError> {
        ensure_request_matches_source_cubemap(request, cubemap)?;

        let path = self.source_cubemap_path(request);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                IblSourceCubemapStagingError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        let texture = texture_asset_from_source_cubemap_zcube(uri, cubemap);
        let TexturePayload::Container { bytes, .. } = texture.payload else {
            return Err(IblSourceCubemapStagingError::UnexpectedZcubePayload);
        };
        let encoded_len = bytes.len();
        let payload_len = encoded_len.saturating_sub(ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE);
        fs::write(&path, bytes).map_err(|source| IblSourceCubemapStagingError::Write {
            path: path.clone(),
            source,
        })?;

        Ok(IblSourceCubemapZcubeWriteReport {
            path,
            encoded_len,
            payload_len,
        })
    }

    pub fn read_source_cubemap_zcube(
        &self,
        request: &IblBakeArtifactRequest,
        _uri: AssetUri,
    ) -> Result<IblSourceCubemapStagingRead, IblSourceCubemapStagingError> {
        let path = self.source_cubemap_path(request);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(IblSourceCubemapStagingRead::Missing);
            }
            Err(source) => {
                return Err(IblSourceCubemapStagingError::Read { path, source });
            }
        };

        let cubemap = decode_zcube_source_cubemap_bytes(&bytes).map_err(|source| {
            IblSourceCubemapStagingError::DecodeZcube {
                path: path.clone(),
                source,
            }
        })?;
        if cubemap.face_size() != request.source_face_size()
            || cubemap.mip_count() != request.source_mip_count()
        {
            return Err(IblSourceCubemapStagingError::RequestSourceLayoutMismatch {
                request_face_size: request.source_face_size(),
                request_mip_count: request.source_mip_count(),
                source_face_size: cubemap.face_size(),
                source_mip_count: cubemap.mip_count(),
            });
        }
        Ok(IblSourceCubemapStagingRead::Hit(cubemap))
    }

    pub fn read_source_cubemap_environment(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
    ) -> Result<SourceCubemapEnvironment, IblSourceCubemapStagingError> {
        let source = match self.read_source_cubemap_zcube(request, uri)? {
            IblSourceCubemapStagingRead::Hit(source) => source,
            IblSourceCubemapStagingRead::Missing => {
                return Err(IblSourceCubemapStagingError::MissingSourceCubemap);
            }
        };
        let derived = match self
            .asset_derived_store()
            .read_asset_derived_artifact(request)
            .map_err(IblSourceCubemapStagingError::AssetDerived)?
        {
            super::IblBakeArtifactAssetDerivedRead::Hit(blob) => blob,
            super::IblBakeArtifactAssetDerivedRead::Missing => {
                return Err(IblSourceCubemapStagingError::MissingAssetDerived);
            }
            super::IblBakeArtifactAssetDerivedRead::Rejected(source) => {
                return Err(IblSourceCubemapStagingError::RejectedAssetDerived(source));
            }
        };

        let source_chain = build_source_cubemap_from_source_mips(
            source.face_size(),
            source.mip_count(),
            source.texels().to_vec(),
        );
        let environment = SourceCubemapEnvironment::new(
            source_chain,
            request.bake_key().source_revision,
            request.bake_key().source_hash,
        );
        source_cubemap_environment_with_bake_artifact(environment, derived.payload())
            .map_err(IblSourceCubemapStagingError::ApplyAssetDerived)
    }

    pub fn write_source_cubemap_staged_bundle(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<IblSourceCubemapStagedBundleReport, IblSourceCubemapStagingError> {
        let source_zcube = self.write_source_cubemap_zcube(request, uri, cubemap)?;
        let asset_derived = self
            .asset_derived_store()
            .write_source_cubemap_asset_derived_artifact(request, cubemap, irradiance_cube)
            .map_err(IblSourceCubemapStagingError::AssetDerived)?;
        Ok(IblSourceCubemapStagedBundleReport {
            source_zcube,
            asset_derived,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IblSourceCubemapStagingRead {
    Hit(ZcubeSourceCubemap),
    Missing,
}

impl IblSourceCubemapStagingRead {
    pub fn cubemap(&self) -> Option<&ZcubeSourceCubemap> {
        match self {
            Self::Hit(cubemap) => Some(cubemap),
            Self::Missing => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblSourceCubemapZcubeWriteReport {
    path: PathBuf,
    encoded_len: usize,
    payload_len: usize,
}

impl IblSourceCubemapZcubeWriteReport {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }

    pub const fn payload_len(&self) -> usize {
        self.payload_len
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblSourceCubemapStagedBundleReport {
    source_zcube: IblSourceCubemapZcubeWriteReport,
    asset_derived: IblBakeArtifactAssetDerivedWriteReport,
}

impl IblSourceCubemapStagedBundleReport {
    pub const fn source_zcube(&self) -> &IblSourceCubemapZcubeWriteReport {
        &self.source_zcube
    }

    pub const fn asset_derived(&self) -> &IblBakeArtifactAssetDerivedWriteReport {
        &self.asset_derived
    }
}

#[derive(Debug, Error)]
pub enum IblSourceCubemapStagingError {
    #[error(
        "source cubemap layout does not match IBL bake request: request face_size={request_face_size}, mip_count={request_mip_count}; source face_size={source_face_size}, mip_count={source_mip_count}"
    )]
    RequestSourceLayoutMismatch {
        request_face_size: u32,
        request_mip_count: u32,
        source_face_size: u32,
        source_mip_count: u32,
    },
    #[error("create staged source cubemap directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("write staged source cubemap .zcube {path:?}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("read staged source cubemap .zcube {path:?}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("decode staged source cubemap .zcube {path:?}: {source}")]
    DecodeZcube {
        path: PathBuf,
        #[source]
        source: ZcubeSourceCubemapError,
    },
    #[error("source cubemap .zcube helper returned a non-container payload")]
    UnexpectedZcubePayload,
    #[error("staged source cubemap .zcube is missing")]
    MissingSourceCubemap,
    #[error("staged asset-derived .zribl is missing")]
    MissingAssetDerived,
    #[error("staged asset-derived .zribl was rejected: {0:?}")]
    RejectedAssetDerived(crate::core::framework::render::IblBakeArtifactBlobError),
    #[error("apply staged asset-derived .zribl to source cubemap: {0:?}")]
    ApplyAssetDerived(SourceCubemapBakeArtifactError),
    #[error("write staged asset-derived .zribl: {0}")]
    AssetDerived(#[source] IblBakeArtifactAssetDerivedError),
}

fn ensure_request_matches_source_cubemap(
    request: &IblBakeArtifactRequest,
    cubemap: &SourceCubemapMipChain,
) -> Result<(), IblSourceCubemapStagingError> {
    if request.source_face_size() == cubemap.source_face_size()
        && request.source_mip_count() == cubemap.source_mip_count()
        && request.pmrem_face_size() == cubemap.pmrem_face_size()
        && request.pmrem_mip_count() == cubemap.pmrem_mip_count()
    {
        return Ok(());
    }

    Err(IblSourceCubemapStagingError::RequestSourceLayoutMismatch {
        request_face_size: request.source_face_size(),
        request_mip_count: request.source_mip_count(),
        source_face_size: cubemap.source_face_size(),
        source_mip_count: cubemap.source_mip_count(),
    })
}
