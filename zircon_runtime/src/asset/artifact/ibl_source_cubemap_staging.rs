use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::asset::assets::{
    decode_zcube_source_cubemap_bytes, texture_asset_from_source_cubemap_zcube, TexturePayload,
    ZcubeSourceCubemap, ZcubeSourceCubemapError, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
};
use crate::asset::AssetUri;
use crate::core::framework::render::{
    source_cubemap_environment_from_source_mips_with_bake_artifact, IblBakeArtifactBlob,
    IblBakeArtifactRequest, IblBakeKey, SourceCubemapBakeArtifactError, SourceCubemapEnvironment,
    SourceCubemapIrradianceCube, SourceCubemapMipChain, IBL_BAKE_ALGORITHM_VERSION,
};
use crate::core::resource::io::transaction::{
    commit_prepared_files, recover_pending_transactions, DurableCommitDisposition,
    DurableCommitReport, DurableTransactionError, PreparedFileWrite, TransactionFault,
};

mod bundle_recovery;

use bundle_recovery::{
    validate_ibl_bundle_target, IblBundleTarget, IblSourceCubemapBundleRecoveryPolicy,
};

use super::ibl_bake_artifact_asset_derived::{
    IblBakeArtifactAssetDerivedError, IblBakeArtifactAssetDerivedStore,
    IblBakeArtifactAssetDerivedWriteReport, IBL_BAKE_ASSET_DERIVED_DIRECTORY,
};
use super::ibl_bake_artifact_cache::ibl_bake_artifact_request_identity_hash;
use super::ibl_source_cubemap_bundle_manifest::{
    IblSourceCubemapBundleManifest, IblSourceCubemapBundleManifestError, IblSourceImageIdentity,
    IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_DIRECTORY, IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_FILE_NAME,
    IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION,
};
pub const IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY: &str = "render/ibl-source";
pub const IBL_SOURCE_CUBEMAP_STAGING_EXTENSION: &str = "zcube";
pub const IBL_SOURCE_CUBEMAP_STAGING_VERSION: u64 = 2026_08_10_0001;
pub(super) const IBL_SOURCE_CUBEMAP_BUNDLE_JOURNAL_DIRECTORY: &str =
    "render/ibl-source-bundle-journal";
const IBL_SOURCE_CUBEMAP_BUNDLE_TRANSACTION_TAG: &str = "ibl-source-bundle";
const IBL_SOURCE_CUBEMAP_BUNDLE_READ_ATTEMPTS: usize = 3;

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
        let source_hash = ibl_source_cubemap_request_identity_hash(request);
        self.source_cubemap_root()
            .join(source_hash)
            .join(format!("source.{IBL_SOURCE_CUBEMAP_STAGING_EXTENSION}"))
    }

    pub(crate) fn bundle_manifest_path(&self, request: &IblBakeArtifactRequest) -> PathBuf {
        self.bundle_manifest_root()
            .join(ibl_bake_artifact_request_identity_hash(request))
            .join(IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_FILE_NAME)
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
        self.write_source_cubemap_staged_bundle(request, uri, cubemap, None)
            .map(|bundle| bundle.source_zcube)
    }

    pub fn read_source_cubemap_zcube(
        &self,
        request: &IblBakeArtifactRequest,
        _uri: AssetUri,
    ) -> Result<IblSourceCubemapStagingRead, IblSourceCubemapStagingError> {
        self.read_source_cubemap_zcube_with_snapshot_hooks(request, || Ok(()), || Ok(()))
    }

    pub fn read_source_cubemap_environment(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
    ) -> Result<SourceCubemapEnvironment, IblSourceCubemapStagingError> {
        self.read_source_cubemap_environment_with_snapshot_hooks(
            request,
            uri,
            || Ok(()),
            || Ok(()),
            || Ok(()),
            || Ok(()),
        )
    }

    pub(crate) fn current_bundle_manifest_matches(
        &self,
        request: &IblBakeArtifactRequest,
        source_image: IblSourceImageIdentity,
    ) -> Result<bool, IblSourceCubemapStagingError> {
        self.recover_pending_bundle_writes()?;
        let Some((manifest_path, manifest_bytes)) = self.read_bundle_manifest_bytes(request)?
        else {
            return Ok(false);
        };
        let Ok(manifest) = IblSourceCubemapBundleManifest::decode(&manifest_bytes) else {
            return Ok(false);
        };
        if !manifest.matches(request, source_image)
            || !regular_file_has_len(
                &self.source_cubemap_path(request),
                manifest.source().encoded_len(),
            )?
            || !regular_file_has_len(
                &self.asset_derived_store().asset_derived_path(request),
                manifest.derived().encoded_len(),
            )?
        {
            return Ok(false);
        }
        let manifest_is_current = matches!(
            self.read_bundle_manifest_bytes(request)?,
            Some((observed_path, observed_bytes))
                if observed_path == manifest_path && observed_bytes == manifest_bytes
        );
        Ok(manifest_is_current && !self.bundle_publication_is_pending()?)
    }

    fn read_source_cubemap_environment_with_snapshot_hooks<
        BeforeSourceRead,
        AfterSourceRead,
        AfterDerivedRead,
        AfterSourceMissBarrier,
    >(
        &self,
        request: &IblBakeArtifactRequest,
        _uri: AssetUri,
        mut before_source_read: BeforeSourceRead,
        mut after_source_read: AfterSourceRead,
        mut after_derived_read: AfterDerivedRead,
        mut after_source_miss_barrier: AfterSourceMissBarrier,
    ) -> Result<SourceCubemapEnvironment, IblSourceCubemapStagingError>
    where
        BeforeSourceRead: FnMut() -> Result<(), IblSourceCubemapStagingError>,
        AfterSourceRead: FnMut() -> Result<(), IblSourceCubemapStagingError>,
        AfterDerivedRead: FnMut() -> Result<(), IblSourceCubemapStagingError>,
        AfterSourceMissBarrier: FnMut() -> Result<(), IblSourceCubemapStagingError>,
    {
        for _ in 0..IBL_SOURCE_CUBEMAP_BUNDLE_READ_ATTEMPTS {
            self.recover_pending_bundle_writes()?;
            before_source_read()?;
            let Some((_manifest_path, manifest_bytes)) =
                self.read_bundle_manifest_bytes(request)?
            else {
                if self.bundle_manifest_miss_is_settled_after_publication_barrier(
                    request,
                    &mut after_source_miss_barrier,
                )? {
                    return Err(IblSourceCubemapStagingError::MissingBundleManifest);
                }
                continue;
            };
            let manifest = IblSourceCubemapBundleManifest::decode(&manifest_bytes)
                .map_err(IblSourceCubemapStagingError::RejectedBundleManifest)?;
            if !manifest.matches_request(request) {
                return Err(IblSourceCubemapStagingError::BundleManifestRequestMismatch);
            }
            let Some((source_path, source_bytes)) = self.read_source_cubemap_bytes(request)? else {
                if self.source_miss_is_settled_after_publication_barrier(
                    request,
                    &mut after_source_miss_barrier,
                )? {
                    return Err(IblSourceCubemapStagingError::MissingSourceCubemap);
                }
                continue;
            };
            after_source_read()?;
            let Some((_derived_path, derived_bytes)) = self
                .asset_derived_store()
                .read_asset_derived_bytes(request)
                .map_err(IblSourceCubemapStagingError::AssetDerived)?
            else {
                return Err(IblSourceCubemapStagingError::MissingAssetDerived);
            };
            after_derived_read()?;
            let manifest_is_current = matches!(
                self.read_bundle_manifest_bytes(request)?,
                Some((_, observed_bytes)) if observed_bytes == manifest_bytes
            );
            if !manifest_is_current || self.bundle_publication_is_pending()? {
                continue;
            }
            if !manifest.source().matches_bytes(&source_bytes) {
                return Err(IblSourceCubemapStagingError::BundlePayloadStampMismatch {
                    payload: "source.zcube",
                });
            }
            if !manifest.derived().matches_bytes(&derived_bytes) {
                return Err(IblSourceCubemapStagingError::BundlePayloadStampMismatch {
                    payload: "asset-derived.zribl",
                });
            }
            let derived = IblBakeArtifactBlob::decode_current_for_request(request, &derived_bytes)
                .map_err(IblSourceCubemapStagingError::RejectedAssetDerived)?;
            let source = decode_source_cubemap_for_request(request, &source_path, &source_bytes)?;
            let source_face_size = source.face_size();
            let source_mip_count = source.mip_count();
            return source_cubemap_environment_from_source_mips_with_bake_artifact(
                source_face_size,
                source_mip_count,
                source.into_texels(),
                request.bake_key().source_revision,
                request.bake_key().source_hash,
                derived.payload(),
            )
            .map_err(IblSourceCubemapStagingError::ApplyAssetDerived);
        }

        Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication)
    }

    fn read_source_cubemap_zcube_after_recovery(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<IblSourceCubemapStagingRead, IblSourceCubemapStagingError> {
        let Some((path, bytes)) = self.read_source_cubemap_bytes(request)? else {
            return Ok(IblSourceCubemapStagingRead::Missing);
        };
        decode_source_cubemap_for_request(request, &path, &bytes)
            .map(IblSourceCubemapStagingRead::Hit)
    }

    fn read_source_cubemap_zcube_with_snapshot_hooks<BeforeSourceRead, AfterSourceMissBarrier>(
        &self,
        request: &IblBakeArtifactRequest,
        mut before_source_read: BeforeSourceRead,
        mut after_source_miss_barrier: AfterSourceMissBarrier,
    ) -> Result<IblSourceCubemapStagingRead, IblSourceCubemapStagingError>
    where
        BeforeSourceRead: FnMut() -> Result<(), IblSourceCubemapStagingError>,
        AfterSourceMissBarrier: FnMut() -> Result<(), IblSourceCubemapStagingError>,
    {
        for _ in 0..IBL_SOURCE_CUBEMAP_BUNDLE_READ_ATTEMPTS {
            self.recover_pending_bundle_writes()?;
            before_source_read()?;
            let source = self.read_source_cubemap_zcube_after_recovery(request)?;
            if !matches!(&source, IblSourceCubemapStagingRead::Missing) {
                return Ok(source);
            }
            if self.source_miss_is_settled_after_publication_barrier(
                request,
                &mut after_source_miss_barrier,
            )? {
                return Ok(source);
            }
        }

        Err(IblSourceCubemapStagingError::BundleReadChangedDuringPublication)
    }

    fn source_miss_is_settled_after_publication_barrier<AfterSourceMissBarrier>(
        &self,
        request: &IblBakeArtifactRequest,
        after_source_miss_barrier: &mut AfterSourceMissBarrier,
    ) -> Result<bool, IblSourceCubemapStagingError>
    where
        AfterSourceMissBarrier: FnMut() -> Result<(), IblSourceCubemapStagingError>,
    {
        let _owner = self
            .asset_derived_store()
            .acquire_bundle_owner_lock()
            .map_err(IblSourceCubemapStagingError::BundleObservation)?;
        if self.bundle_publication_is_pending()? {
            return Ok(false);
        }
        after_source_miss_barrier()?;
        self.read_source_cubemap_bytes(request)
            .map(|source| source.is_none())
    }

    fn bundle_manifest_miss_is_settled_after_publication_barrier<AfterSourceMissBarrier>(
        &self,
        request: &IblBakeArtifactRequest,
        after_source_miss_barrier: &mut AfterSourceMissBarrier,
    ) -> Result<bool, IblSourceCubemapStagingError>
    where
        AfterSourceMissBarrier: FnMut() -> Result<(), IblSourceCubemapStagingError>,
    {
        let _owner = self
            .asset_derived_store()
            .acquire_bundle_owner_lock()
            .map_err(IblSourceCubemapStagingError::BundleObservation)?;
        if self.bundle_publication_is_pending()? {
            return Ok(false);
        }
        after_source_miss_barrier()?;
        self.read_bundle_manifest_bytes(request)
            .map(|manifest| manifest.is_none())
    }

    fn read_source_cubemap_bytes(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<Option<(PathBuf, Vec<u8>)>, IblSourceCubemapStagingError> {
        let path = self.source_cubemap_path(request);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(source) => {
                return Err(IblSourceCubemapStagingError::Read { path, source });
            }
        };
        Ok(Some((path, bytes)))
    }

    fn read_bundle_manifest_bytes(
        &self,
        request: &IblBakeArtifactRequest,
    ) -> Result<Option<(PathBuf, Vec<u8>)>, IblSourceCubemapStagingError> {
        let path = self.bundle_manifest_path(request);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some((path, bytes))),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(IblSourceCubemapStagingError::ReadBundleManifest { path, source }),
        }
    }

    pub fn write_source_cubemap_staged_bundle(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<IblSourceCubemapStagedBundleReport, IblSourceCubemapStagingError> {
        self.write_source_cubemap_staged_bundle_with_fault(
            request,
            uri,
            cubemap,
            irradiance_cube,
            TransactionFault::None,
        )
    }

    #[cfg(test)]
    fn write_source_cubemap_staged_bundle_for_test(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
        fault: TransactionFault,
    ) -> Result<IblSourceCubemapStagedBundleReport, IblSourceCubemapStagingError> {
        self.write_source_cubemap_staged_bundle_with_fault(
            request,
            uri,
            cubemap,
            irradiance_cube,
            fault,
        )
    }

    fn write_source_cubemap_staged_bundle_with_fault(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
        fault: TransactionFault,
    ) -> Result<IblSourceCubemapStagedBundleReport, IblSourceCubemapStagingError> {
        let (writes, bundle) =
            self.prepare_source_cubemap_staged_bundle(request, uri, cubemap, irradiance_cube)?;
        self.commit_prepared_bundle_writes_with_fault(writes, fault)?;
        Ok(bundle)
    }

    pub(crate) fn prepare_source_cubemap_staged_bundle(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    ) -> Result<
        (Vec<PreparedFileWrite>, IblSourceCubemapStagedBundleReport),
        IblSourceCubemapStagingError,
    > {
        self.prepare_source_cubemap_staged_bundle_with_source_image(
            request,
            uri,
            cubemap,
            irradiance_cube,
            IblSourceImageIdentity::default(),
        )
    }

    pub(crate) fn prepare_source_cubemap_staged_bundle_with_source_image(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
        irradiance_cube: Option<&SourceCubemapIrradianceCube>,
        source_image: IblSourceImageIdentity,
    ) -> Result<
        (Vec<PreparedFileWrite>, IblSourceCubemapStagedBundleReport),
        IblSourceCubemapStagingError,
    > {
        self.recover_pending_bundle_writes()?;
        let source_zcube = self.prepare_source_cubemap_zcube(request, uri, cubemap)?;
        let asset_derived = self
            .asset_derived_store()
            .prepare_source_cubemap_asset_derived_artifact(request, cubemap, irradiance_cube)
            .map_err(IblSourceCubemapStagingError::AssetDerived)?;
        let (source_path, source_bytes, source_zcube) = source_zcube.into_parts();
        let (asset_derived_path, asset_derived_bytes, asset_derived) = asset_derived.into_parts();
        let manifest_path = self.bundle_manifest_path(request);
        let manifest_bytes = IblSourceCubemapBundleManifest::new(
            request,
            source_image,
            &source_bytes,
            &asset_derived_bytes,
        )
        .encode()
        .to_vec();
        Ok((
            vec![
                PreparedFileWrite::new(source_path, source_bytes),
                PreparedFileWrite::new(asset_derived_path, asset_derived_bytes),
                PreparedFileWrite::new(manifest_path, manifest_bytes),
            ],
            IblSourceCubemapStagedBundleReport {
                source_zcube,
                asset_derived,
            },
        ))
    }

    pub(crate) fn prepare_bundle_manifest_for_existing_source(
        &self,
        request: &IblBakeArtifactRequest,
        source_image: IblSourceImageIdentity,
        asset_derived_bytes: &[u8],
    ) -> Result<PreparedFileWrite, IblSourceCubemapStagingError> {
        let source_path = self.source_cubemap_path(request);
        let source_bytes =
            fs::read(&source_path).map_err(|source| IblSourceCubemapStagingError::Read {
                path: source_path,
                source,
            })?;
        let bytes = IblSourceCubemapBundleManifest::new(
            request,
            source_image,
            &source_bytes,
            asset_derived_bytes,
        )
        .encode()
        .to_vec();
        Ok(PreparedFileWrite::new(
            self.bundle_manifest_path(request),
            bytes,
        ))
    }

    pub(crate) fn commit_prepared_bundle_writes(
        &self,
        writes: Vec<PreparedFileWrite>,
    ) -> Result<(), IblSourceCubemapStagingError> {
        self.commit_prepared_bundle_writes_with_fault(writes, TransactionFault::None)
    }

    pub(crate) fn validate_bundle_target(&self, target: &Path) -> Result<(), String> {
        validate_ibl_bundle_target(
            &self.source_cubemap_root(),
            target,
            IblBundleTarget::Source,
        )
        .or_else(|source_error| {
            validate_ibl_bundle_target(
                &self.asset_derived_root(),
                target,
                IblBundleTarget::AssetDerived,
            )
            .or_else(|asset_derived_error| {
                validate_ibl_bundle_target(
                    &self.bundle_manifest_root(),
                    target,
                    IblBundleTarget::Manifest,
                )
                .map_err(|manifest_error| {
                    format!(
                        "IBL source bundle target {} is invalid: source ({source_error}); asset-derived ({asset_derived_error}); manifest ({manifest_error})",
                        target.display()
                    )
                })
            })
        })
    }

    fn commit_prepared_bundle_writes_with_fault(
        &self,
        writes: Vec<PreparedFileWrite>,
        fault: TransactionFault,
    ) -> Result<(), IblSourceCubemapStagingError> {
        if writes.is_empty() {
            return Ok(());
        }
        self.recover_pending_bundle_writes()?;
        let mut commit_report = DurableCommitReport::default();
        let disposition = commit_prepared_files(
            &self.bundle_journal_directory(),
            IBL_SOURCE_CUBEMAP_BUNDLE_TRANSACTION_TAG,
            writes,
            fault,
            &mut commit_report,
        )
        .map_err(IblSourceCubemapStagingError::BundlePublication)?;
        if disposition == DurableCommitDisposition::CommitRecoveryDeferred {
            return Err(IblSourceCubemapStagingError::BundleCommitRecoveryDeferred);
        }
        Ok(())
    }

    pub(crate) fn prepare_source_cubemap_zcube(
        &self,
        request: &IblBakeArtifactRequest,
        uri: AssetUri,
        cubemap: &SourceCubemapMipChain,
    ) -> Result<PreparedIblSourceCubemapZcubeWrite, IblSourceCubemapStagingError> {
        ensure_request_matches_source_cubemap(request, cubemap)?;
        let path = self.source_cubemap_path(request);
        let texture = texture_asset_from_source_cubemap_zcube(uri, cubemap);
        let TexturePayload::Container { bytes, .. } = texture.payload else {
            return Err(IblSourceCubemapStagingError::UnexpectedZcubePayload);
        };
        let encoded_len = bytes.len();
        let payload_len = encoded_len.saturating_sub(ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE);
        Ok(PreparedIblSourceCubemapZcubeWrite {
            path: path.clone(),
            bytes,
            report: IblSourceCubemapZcubeWriteReport {
                path,
                encoded_len,
                payload_len,
            },
        })
    }

    fn source_cubemap_root(&self) -> PathBuf {
        self.cache_root
            .join(IBL_SOURCE_CUBEMAP_STAGING_DIRECTORY)
            .join(format!("v{:016x}", IBL_SOURCE_CUBEMAP_STAGING_VERSION))
    }

    fn asset_derived_root(&self) -> PathBuf {
        self.cache_root
            .join(IBL_BAKE_ASSET_DERIVED_DIRECTORY)
            .join(format!("v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
    }

    fn bundle_manifest_root(&self) -> PathBuf {
        self.cache_root
            .join(IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_DIRECTORY)
            .join(format!(
                "v{:016x}-{:08x}",
                IBL_SOURCE_CUBEMAP_STAGING_VERSION,
                IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SCHEMA_VERSION
            ))
    }

    fn bundle_journal_directory(&self) -> PathBuf {
        self.cache_root
            .join(IBL_SOURCE_CUBEMAP_BUNDLE_JOURNAL_DIRECTORY)
            .join(format!(
                "source-v{:016x}",
                IBL_SOURCE_CUBEMAP_STAGING_VERSION
            ))
            .join(format!("derived-v{:016x}", IBL_BAKE_ALGORITHM_VERSION))
    }

    fn recover_pending_bundle_writes(&self) -> Result<(), IblSourceCubemapStagingError> {
        let journal_directory = self.bundle_journal_directory();
        // A fresh cache has no journal directory yet. The durable transaction
        // layer creates its owner lock in the journal's parent, so avoid
        // acquiring that lock until a previous publication could exist.
        if !journal_directory.exists() {
            return Ok(());
        }
        let resolved_journal_directory =
            fs::canonicalize(&journal_directory).map_err(|source| {
                IblSourceCubemapStagingError::BundleJournalRead {
                    path: journal_directory.clone(),
                    source,
                }
            })?;
        let mut policy = IblSourceCubemapBundleRecoveryPolicy::new(
            resolved_journal_directory,
            self.source_cubemap_root(),
            self.asset_derived_root(),
            self.bundle_manifest_root(),
        );
        recover_pending_transactions(
            &journal_directory,
            IBL_SOURCE_CUBEMAP_BUNDLE_TRANSACTION_TAG,
            &mut policy,
        )
        .map_err(IblSourceCubemapStagingError::BundleRecovery)?;
        Ok(())
    }

    fn bundle_publication_is_pending(&self) -> Result<bool, IblSourceCubemapStagingError> {
        let journal_directory = self.bundle_journal_directory();
        let mut entries = match fs::read_dir(&journal_directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(source) => {
                return Err(IblSourceCubemapStagingError::BundleJournalRead {
                    path: journal_directory,
                    source,
                });
            }
        };
        entries
            .next()
            .transpose()
            .map(|entry| entry.is_some())
            .map_err(|source| IblSourceCubemapStagingError::BundleJournalRead {
                path: journal_directory,
                source,
            })
    }
}

pub(crate) struct PreparedIblSourceCubemapZcubeWrite {
    path: PathBuf,
    bytes: Vec<u8>,
    report: IblSourceCubemapZcubeWriteReport,
}

impl PreparedIblSourceCubemapZcubeWrite {
    pub(crate) fn into_parts(self) -> (PathBuf, Vec<u8>, IblSourceCubemapZcubeWriteReport) {
        (self.path, self.bytes, self.report)
    }
}

fn regular_file_has_len(
    path: &Path,
    expected_len: u64,
) -> Result<bool, IblSourceCubemapStagingError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(IblSourceCubemapStagingError::InspectBundlePayload {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    Ok(!metadata.file_type().is_symlink() && metadata.is_file() && metadata.len() == expected_len)
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
    #[error("staged IBL source bundle manifest is missing")]
    MissingBundleManifest,
    #[error("read staged IBL source bundle manifest {path:?}: {source}")]
    ReadBundleManifest {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("staged IBL source bundle manifest was rejected: {0}")]
    RejectedBundleManifest(#[source] IblSourceCubemapBundleManifestError),
    #[error("staged IBL source bundle manifest does not match the requested generation")]
    BundleManifestRequestMismatch,
    #[error("staged IBL source bundle payload stamp does not match {payload}")]
    BundlePayloadStampMismatch { payload: &'static str },
    #[error("inspect staged IBL source bundle payload {path:?}: {source}")]
    InspectBundlePayload {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("staged asset-derived .zribl was rejected: {0:?}")]
    RejectedAssetDerived(crate::core::framework::render::IblBakeArtifactBlobError),
    #[error("apply staged asset-derived .zribl to source cubemap: {0:?}")]
    ApplyAssetDerived(SourceCubemapBakeArtifactError),
    #[error("recover staged IBL source bundle publication: {0}")]
    BundleRecovery(#[source] DurableTransactionError),
    #[error("publish staged IBL source bundle: {0}")]
    BundlePublication(#[source] DurableTransactionError),
    #[error("IBL source bundle commit needs recovery before the next read or write")]
    BundleCommitRecoveryDeferred,
    #[error("acquire IBL source bundle observation owner: {0}")]
    BundleObservation(#[source] IblBakeArtifactAssetDerivedError),
    #[error("read IBL source bundle journal {path:?}: {source}")]
    BundleJournalRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("IBL source bundle changed during a coherent read")]
    BundleReadChangedDuringPublication,
    #[error("write staged asset-derived .zribl: {0}")]
    AssetDerived(#[source] IblBakeArtifactAssetDerivedError),
}

fn decode_source_cubemap_for_request(
    request: &IblBakeArtifactRequest,
    path: &Path,
    bytes: &[u8],
) -> Result<ZcubeSourceCubemap, IblSourceCubemapStagingError> {
    let cubemap = decode_zcube_source_cubemap_bytes(bytes).map_err(|source| {
        IblSourceCubemapStagingError::DecodeZcube {
            path: path.to_path_buf(),
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
    Ok(cubemap)
}

fn ensure_request_matches_source_cubemap(
    request: &IblBakeArtifactRequest,
    cubemap: &SourceCubemapMipChain,
) -> Result<(), IblSourceCubemapStagingError> {
    if request.source_face_size() == cubemap.source_face_size()
        && request.source_mip_count() == cubemap.source_mip_count()
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

fn ibl_source_cubemap_request_identity_hash(request: &IblBakeArtifactRequest) -> String {
    let mut hasher = blake3::Hasher::new();
    update_bake_key_hash(&mut hasher, request.bake_key());
    hasher.update(&request.source_face_size().to_le_bytes());
    hasher.update(&request.source_mip_count().to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

fn update_bake_key_hash(hasher: &mut blake3::Hasher, bake_key: IblBakeKey) {
    hasher.update(&bake_key.source_kind.to_le_bytes());
    hasher.update(&bake_key.source_revision.to_le_bytes());
    for values in [
        bake_key.horizon_color,
        bake_key.zenith_color,
        bake_key.ground_color,
        bake_key.source_hash,
    ] {
        for value in values {
            hasher.update(&value.to_le_bytes());
        }
    }
}

#[cfg(test)]
#[path = "ibl_source_cubemap_staging/tests.rs"]
mod tests;
