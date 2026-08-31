use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::asset::AssetUri;
use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactBlobCandidate, IblBakeArtifactBlobError,
    IblBakeArtifactDescriptor, IblBakeArtifactPayload, IblBakeArtifactPayloadError,
    IblBakeArtifactProducer, IblBakeArtifactRequest, SourceCubemapIrradianceCube,
    SourceCubemapMipChain, IBL_BAKE_ALGORITHM_VERSION,
};
use crate::core::resource::io::{atomic_write, sync_parent_directory};

use super::ibl_bake_artifact_cache::ibl_bake_artifact_request_identity_hash;
use super::ibl_source_cubemap_staging::{
    IblSourceCubemapStagingError, IblSourceCubemapStagingStore,
    IBL_SOURCE_CUBEMAP_BUNDLE_JOURNAL_DIRECTORY,
};

pub const IBL_BAKE_ASSET_DERIVED_DIRECTORY: &str = "render/ibl-derived";
pub const IBL_BAKE_ASSET_DERIVED_EXTENSION: &str = "zribl";
const IBL_BAKE_ASSET_DERIVED_BUNDLE_URI: &str = "res://generated/ibl/asset-derived.zcube";

pub(crate) struct PreparedIblBakeArtifactAssetDerivedWrite {
    path: PathBuf,
    bytes: Vec<u8>,
    report: IblBakeArtifactAssetDerivedWriteReport,
}

impl PreparedIblBakeArtifactAssetDerivedWrite {
    pub(crate) fn into_parts(self) -> (PathBuf, Vec<u8>, IblBakeArtifactAssetDerivedWriteReport) {
        (self.path, self.bytes, self.report)
    }
}

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
        let _owner = self.acquire_bundle_owner_lock()?;
        self.reject_blob_write_for_paired_source(blob.descriptor())?;
        let prepared = self.prepare_asset_derived_blob(blob)?;
        atomic_write(&prepared.path, &prepared.bytes).map_err(|source| {
            IblBakeArtifactAssetDerivedError::Write {
                path: prepared.path.clone(),
                source,
            }
        })?;
        Ok(prepared.report)
    }

    pub(crate) fn prepare_source_cubemap_asset_derived_artifact(
        &self,
        request: &IblBakeArtifactRequest,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<PreparedIblBakeArtifactAssetDerivedWrite, IblBakeArtifactAssetDerivedError> {
        let descriptor = IblBakeArtifactDescriptor::current_for_request(request);
        let payload =
            IblBakeArtifactPayload::from_source_cubemap(descriptor, cubemap, irradiance_cube)
                .map_err(|error| IblBakeArtifactAssetDerivedError::Payload { error })?;
        self.prepare_asset_derived_blob(&IblBakeArtifactBlob::from_payload(payload))
    }

    fn prepare_asset_derived_blob(
        &self,
        blob: &IblBakeArtifactBlob,
    ) -> Result<PreparedIblBakeArtifactAssetDerivedWrite, IblBakeArtifactAssetDerivedError> {
        if blob.descriptor().producer() != IblBakeArtifactProducer::AssetImporterCpu {
            return Err(IblBakeArtifactAssetDerivedError::InvalidProducer {
                producer: blob.descriptor().producer(),
            });
        }
        let path = self.asset_derived_path_for_descriptor(blob.descriptor());
        let bytes = blob.encode();
        let report = IblBakeArtifactAssetDerivedWriteReport {
            path,
            descriptor: blob.descriptor(),
            encoded_len: bytes.len(),
            payload_len: blob.payload().bytes().len(),
        };
        Ok(PreparedIblBakeArtifactAssetDerivedWrite {
            path: report.path.clone(),
            bytes,
            report,
        })
    }

    /// Publishes a recipe-keyed artifact through its source bundle transaction.
    pub fn write_source_cubemap_asset_derived_artifact(
        &self,
        request: &IblBakeArtifactRequest,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<IblBakeArtifactAssetDerivedWriteReport, IblBakeArtifactAssetDerivedError> {
        let uri = AssetUri::parse(IBL_BAKE_ASSET_DERIVED_BUNDLE_URI)
            .expect("the built-in asset-derived bundle URI must remain valid");
        IblSourceCubemapStagingStore::new(self.cache_root.clone())
            .write_source_cubemap_staged_bundle(request, uri, cubemap, irradiance_cube)
            .map(|bundle| bundle.asset_derived().clone())
            .map_err(|source| IblBakeArtifactAssetDerivedError::BundlePublication(Box::new(source)))
    }

    pub fn read_asset_derived_artifact(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<IblBakeArtifactAssetDerivedRead, IblBakeArtifactAssetDerivedError> {
        let Some((_path, bytes)) = self.read_asset_derived_bytes(request)? else {
            return Ok(IblBakeArtifactAssetDerivedRead::Missing);
        };
        Ok(
            match IblBakeArtifactBlob::decode_current_for_request(request, &bytes) {
                Ok(blob) => IblBakeArtifactAssetDerivedRead::Hit(blob),
                Err(error) => IblBakeArtifactAssetDerivedRead::Rejected(error),
            },
        )
    }

    pub(crate) fn read_asset_derived_bytes(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<Option<(PathBuf, Vec<u8>)>, IblBakeArtifactAssetDerivedError> {
        let path = self.asset_derived_path(request);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some((path, bytes))),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(IblBakeArtifactAssetDerivedError::Read { path, source }),
        }
    }

    fn reject_blob_write_for_paired_source(
        &self,
        descriptor: IblBakeArtifactDescriptor,
    ) -> Result<(), IblBakeArtifactAssetDerivedError> {
        let request = request_for_descriptor(descriptor);
        let source_path = IblSourceCubemapStagingStore::new(self.cache_root.clone())
            .source_cubemap_path(&request);
        match fs::symlink_metadata(&source_path) {
            Ok(_) => Err(
                IblBakeArtifactAssetDerivedError::PairedSourceRequiresBundle { path: source_path },
            ),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(IblBakeArtifactAssetDerivedError::InspectPairedSource {
                path: source_path,
                source,
            }),
        }
    }

    pub(super) fn acquire_bundle_owner_lock(
        &self,
    ) -> Result<IblBakeArtifactBundleOwnerLock, IblBakeArtifactAssetDerivedError> {
        let journal_directory = self.bundle_journal_directory();
        if !journal_directory.exists() {
            fs::create_dir_all(&journal_directory).map_err(|source| {
                IblBakeArtifactAssetDerivedError::PrepareBundleOwner {
                    path: journal_directory.clone(),
                    source,
                }
            })?;
        }
        let metadata = fs::symlink_metadata(&journal_directory).map_err(|source| {
            IblBakeArtifactAssetDerivedError::PrepareBundleOwner {
                path: journal_directory.clone(),
                source,
            }
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(IblBakeArtifactAssetDerivedError::InvalidBundleOwner {
                path: journal_directory,
            });
        }
        IblBakeArtifactBundleOwnerLock::acquire(&journal_directory)
    }

    fn bundle_journal_directory(&self) -> PathBuf {
        self.cache_root
            .join(IBL_SOURCE_CUBEMAP_BUNDLE_JOURNAL_DIRECTORY)
            .join(format!(
                "source-v{:016x}",
                super::ibl_source_cubemap_staging::IBL_SOURCE_CUBEMAP_STAGING_VERSION
            ))
            .join(format!("derived-v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
    }
}

pub(super) struct IblBakeArtifactBundleOwnerLock {
    file: File,
}

impl IblBakeArtifactBundleOwnerLock {
    fn acquire(journal_directory: &Path) -> Result<Self, IblBakeArtifactAssetDerivedError> {
        let parent = journal_directory.parent().ok_or_else(|| {
            IblBakeArtifactAssetDerivedError::InvalidBundleOwner {
                path: journal_directory.to_path_buf(),
            }
        })?;
        let name = journal_directory
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| IblBakeArtifactAssetDerivedError::InvalidBundleOwner {
                path: journal_directory.to_path_buf(),
            })?;
        let path = parent.join(format!(".{name}.zrlock"));
        let (file, created) = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => (file, true),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let metadata = fs::symlink_metadata(&path).map_err(|source| {
                    IblBakeArtifactAssetDerivedError::PrepareBundleOwner {
                        path: path.clone(),
                        source,
                    }
                })?;
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(IblBakeArtifactAssetDerivedError::InvalidBundleOwner { path });
                }
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&path)
                    .map_err(
                        |source| IblBakeArtifactAssetDerivedError::PrepareBundleOwner {
                            path: path.clone(),
                            source,
                        },
                    )?;
                (file, false)
            }
            Err(source) => {
                return Err(IblBakeArtifactAssetDerivedError::PrepareBundleOwner { path, source });
            }
        };
        if created {
            file.sync_all()
                .and_then(|()| sync_parent_directory(&path))
                .map_err(
                    |source| IblBakeArtifactAssetDerivedError::PrepareBundleOwner {
                        path: path.clone(),
                        source,
                    },
                )?;
        }
        File::try_lock(&file).map_err(|source| {
            let source = match source {
                TryLockError::WouldBlock => io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "a source/derived IBL bundle publication is already in progress",
                ),
                TryLockError::Error(source) => source,
            };
            IblBakeArtifactAssetDerivedError::BundleOwnerBusy { path, source }
        })?;
        Ok(Self { file })
    }
}

impl Drop for IblBakeArtifactBundleOwnerLock {
    fn drop(&mut self) {
        let _ = File::unlock(&self.file);
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
    #[error("asset-derived IBL artifact requires an importer CPU artifact, got {producer:?}")]
    InvalidProducer { producer: IblBakeArtifactProducer },
    #[error("asset-derived blob cannot replace a paired source bundle at {path:?}")]
    PairedSourceRequiresBundle { path: PathBuf },
    #[error("prepare IBL source bundle owner {path:?}: {source}")]
    PrepareBundleOwner {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("IBL source bundle owner must be a real directory or regular lock file: {path:?}")]
    InvalidBundleOwner { path: PathBuf },
    #[error("acquire IBL source bundle owner {path:?}: {source}")]
    BundleOwnerBusy {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("inspect paired source bundle {path:?}: {source}")]
    InspectPairedSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
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
    #[error("publish source/derived IBL bundle: {0}")]
    BundlePublication(#[source] Box<IblSourceCubemapStagingError>),
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

#[cfg(test)]
mod tests {
    #[test]
    fn asset_derived_writer_uses_runtime_atomic_publication() {
        let source = include_str!("ibl_bake_artifact_asset_derived.rs");
        let writer = source
            .split("pub fn write_asset_derived_blob(")
            .nth(1)
            .and_then(|writer| {
                writer
                    .split("pub fn write_source_cubemap_asset_derived_artifact(")
                    .next()
            })
            .expect("asset-derived store must retain its writer");

        assert!(source.contains("core::resource::io::atomic_write"));
        assert!(writer.contains("atomic_write("));
        assert!(!writer.contains("fs::write("));
    }
}
