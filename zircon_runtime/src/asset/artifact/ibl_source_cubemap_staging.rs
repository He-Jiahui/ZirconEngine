use std::fs;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

use crate::asset::assets::{
    decode_zcube_source_cubemap_bytes, texture_asset_from_source_cubemap_zcube, TexturePayload,
    ZcubeSourceCubemap, ZcubeSourceCubemapError, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
};
use crate::asset::AssetUri;
use crate::core::framework::render::{
    source_cubemap_environment_from_source_mips_with_bake_artifact, IblBakeArtifactRequest,
    IblBakeKey, SourceCubemapBakeArtifactError, SourceCubemapEnvironment,
    SourceCubemapIrradianceCube, SourceCubemapMipChain, IBL_BAKE_ALGORITHM_VERSION,
};
use crate::core::resource::io::transaction::{
    commit_prepared_files, recover_pending_transactions, DurableCommitDisposition,
    DurableCommitReport, DurableTransactionError, JournalDocument, PreparedFileWrite,
    RecoveryPolicy, TransactionFault,
};

use super::ibl_bake_artifact_asset_derived::{
    IblBakeArtifactAssetDerivedError, IblBakeArtifactAssetDerivedStore,
    IblBakeArtifactAssetDerivedWriteReport, IBL_BAKE_ASSET_DERIVED_DIRECTORY,
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
            let derived = self
                .asset_derived_store()
                .read_asset_derived_artifact(request)
                .map_err(IblSourceCubemapStagingError::AssetDerived)?;
            after_derived_read()?;
            // Target replacement is sequential inside the durable transaction. The
            // repeated source read and journal check reject a cross-generation view.
            let source_is_current = matches!(
                self.read_source_cubemap_bytes(request)?,
                Some((_, observed_bytes)) if observed_bytes == source_bytes
            );
            let derived_is_current = self
                .asset_derived_store()
                .read_asset_derived_artifact(request)
                .map_err(IblSourceCubemapStagingError::AssetDerived)?
                == derived;
            if !source_is_current || !derived_is_current || self.bundle_publication_is_pending()? {
                continue;
            }

            let derived = match derived {
                super::IblBakeArtifactAssetDerivedRead::Hit(blob) => blob,
                super::IblBakeArtifactAssetDerivedRead::Missing => {
                    return Err(IblSourceCubemapStagingError::MissingAssetDerived);
                }
                super::IblBakeArtifactAssetDerivedRead::Rejected(source) => {
                    return Err(IblSourceCubemapStagingError::RejectedAssetDerived(source));
                }
            };

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
        self.recover_pending_bundle_writes()?;
        let source_zcube = self.prepare_source_cubemap_zcube(request, uri, cubemap)?;
        let asset_derived = self
            .asset_derived_store()
            .prepare_source_cubemap_asset_derived_artifact(request, cubemap, irradiance_cube)
            .map_err(IblSourceCubemapStagingError::AssetDerived)?;
        let (source_path, source_bytes, source_zcube) = source_zcube.into_parts();
        let (asset_derived_path, asset_derived_bytes, asset_derived) = asset_derived.into_parts();
        let mut commit_report = DurableCommitReport::default();
        let disposition = commit_prepared_files(
            &self.bundle_journal_directory(),
            IBL_SOURCE_CUBEMAP_BUNDLE_TRANSACTION_TAG,
            vec![
                PreparedFileWrite::new(source_path, source_bytes),
                PreparedFileWrite::new(asset_derived_path, asset_derived_bytes),
            ],
            fault,
            &mut commit_report,
        )
        .map_err(IblSourceCubemapStagingError::BundlePublication)?;
        if disposition == DurableCommitDisposition::CommitRecoveryDeferred {
            return Err(IblSourceCubemapStagingError::BundleCommitRecoveryDeferred);
        }
        Ok(IblSourceCubemapStagedBundleReport {
            source_zcube,
            asset_derived,
        })
    }

    fn prepare_source_cubemap_zcube(
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
        let mut policy = IblSourceCubemapBundleRecoveryPolicy {
            journal_directory: journal_directory.clone(),
            source_root: self.source_cubemap_root(),
            asset_derived_root: self.asset_derived_root(),
        };
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

struct PreparedIblSourceCubemapZcubeWrite {
    path: PathBuf,
    bytes: Vec<u8>,
    report: IblSourceCubemapZcubeWriteReport,
}

impl PreparedIblSourceCubemapZcubeWrite {
    fn into_parts(self) -> (PathBuf, Vec<u8>, IblSourceCubemapZcubeWriteReport) {
        (self.path, self.bytes, self.report)
    }
}

struct IblSourceCubemapBundleRecoveryPolicy {
    journal_directory: PathBuf,
    source_root: PathBuf,
    asset_derived_root: PathBuf,
}

impl RecoveryPolicy for IblSourceCubemapBundleRecoveryPolicy {
    fn validate_document(
        &self,
        journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        if journal_path.parent() != Some(self.journal_directory.as_path()) {
            return Err(format!(
                "IBL source bundle journal is outside its configured directory: {}",
                journal_path.display()
            ));
        }
        if document.retired_path().is_some() {
            return Err("IBL source bundle transactions cannot retire live files".to_owned());
        }
        validate_ibl_bundle_target(&self.source_root, document.target(), IblBundleTarget::Source)
            .or_else(|source_error| {
                validate_ibl_bundle_target(
                    &self.asset_derived_root,
                    document.target(),
                    IblBundleTarget::AssetDerived,
                )
                .map_err(|asset_derived_error| {
                    format!(
                        "IBL source bundle target {} is neither a source cubemap ({source_error}) nor an asset-derived artifact ({asset_derived_error})",
                        document.target().display()
                    )
                })
            })
    }
}

#[derive(Clone, Copy)]
enum IblBundleTarget {
    Source,
    AssetDerived,
}

fn validate_ibl_bundle_target(
    root: &Path,
    target: &Path,
    kind: IblBundleTarget,
) -> Result<(), String> {
    validate_ibl_bundle_directory(root)?;
    let relative = target.strip_prefix(root).map_err(|_| {
        format!(
            "target {} does not reside under {}",
            target.display(),
            root.display()
        )
    })?;
    let components = relative.components().collect::<Vec<_>>();
    let [Component::Normal(identity), Component::Normal(file_name)] = components.as_slice() else {
        return Err("target must have exactly an identity directory and artifact file".to_owned());
    };
    let identity = identity
        .to_str()
        .ok_or_else(|| "target identity is not valid Unicode".to_owned())?;
    if identity.len() != blake3::OUT_LEN * 2
        || !identity
            .bytes()
            .all(|value| value.is_ascii_digit() || matches!(value, b'a'..=b'f'))
    {
        return Err("target identity is not a lowercase BLAKE3 digest".to_owned());
    }
    let file_name = file_name
        .to_str()
        .ok_or_else(|| "target filename is not valid Unicode".to_owned())?;
    match kind {
        IblBundleTarget::Source if file_name == "source.zcube" => {}
        IblBundleTarget::AssetDerived if valid_asset_derived_filename(file_name) => {}
        IblBundleTarget::Source => {
            return Err("source target filename is not source.zcube".to_owned())
        }
        IblBundleTarget::AssetDerived => {
            return Err("asset-derived target filename has invalid face/mip layout".to_owned())
        }
    }
    validate_ibl_bundle_directory(&root.join(identity))
}

fn valid_asset_derived_filename(file_name: &str) -> bool {
    let Some(layout) = file_name
        .strip_prefix("face_")
        .and_then(|value| value.strip_suffix(".zribl"))
        .and_then(|value| value.split_once("_mips_"))
    else {
        return false;
    };
    let (face_size, mip_count) = layout;
    face_size.len() >= 4
        && mip_count.len() >= 2
        && face_size.parse::<u32>().is_ok_and(|value| value > 0)
        && mip_count.parse::<u32>().is_ok_and(|value| value > 0)
}

fn validate_ibl_bundle_directory(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        // The durable journal is written before target staging creates its
        // parent directories. A process can stop in that interval, and the
        // next read must still be able to remove the uncommitted intent.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("inspect directory {}: {error}", path.display())),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{} is not a real directory", path.display()));
    }
    Ok(())
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
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::asset::AssetUri;
    use crate::core::framework::render::{
        IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
        IblBakeArtifactPayload, IblBakeArtifactRequest, IblBakeKey, SourceCubemapMipChain,
        SourceCubemapPrefilterQuality,
    };
    use crate::core::math::Real;
    use crate::core::resource::io::transaction::TransactionFault;

    use super::{
        IblBakeArtifactAssetDerivedError, IblSourceCubemapStagingRead, IblSourceCubemapStagingStore,
    };

    static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn source_path_ignores_pmrem_layout_and_optional_derived_contents() {
        let store = IblSourceCubemapStagingStore::new("E:/cache");
        let source = request(
            7,
            [11; 4],
            256,
            9,
            128,
            8,
            IblBakeArtifactContents::PMREM_SH9,
        );
        let alternate_recipe = request(
            7,
            [11; 4],
            256,
            9,
            64,
            7,
            IblBakeArtifactContents::PMREM_SH9_IEM,
        );

        assert_eq!(
            store.source_cubemap_path(&source),
            store.source_cubemap_path(&alternate_recipe)
        );
        assert_ne!(
            store.asset_derived_store().asset_derived_path(&source),
            store
                .asset_derived_store()
                .asset_derived_path(&alternate_recipe)
        );
    }

    #[test]
    fn source_path_changes_with_source_identity_or_layout() {
        let store = IblSourceCubemapStagingStore::new(Path::new("E:/cache"));
        let source = request(
            7,
            [11; 4],
            256,
            9,
            128,
            8,
            IblBakeArtifactContents::PMREM_SH9,
        );
        let changed_bytes = request(
            8,
            [12; 4],
            256,
            9,
            128,
            8,
            IblBakeArtifactContents::PMREM_SH9,
        );
        let changed_layout = request(
            7,
            [11; 4],
            512,
            10,
            128,
            8,
            IblBakeArtifactContents::PMREM_SH9,
        );

        assert_ne!(
            store.source_cubemap_path(&source),
            store.source_cubemap_path(&changed_bytes)
        );
        assert_ne!(
            store.source_cubemap_path(&source),
            store.source_cubemap_path(&changed_layout)
        );
    }

    #[test]
    fn source_zcube_writer_delegates_to_bundle_publication() {
        let source = include_str!("ibl_source_cubemap_staging.rs");
        let writer = source
            .split("pub fn write_source_cubemap_zcube(")
            .nth(1)
            .and_then(|writer| writer.split("pub fn read_source_cubemap_zcube(").next())
            .expect("source staging must retain the compatibility writer");

        assert!(writer.contains("write_source_cubemap_staged_bundle("));
        assert!(!writer.contains("PreparedFileWrite::new("));
        assert!(!writer.contains("atomic_write("));
        assert!(!writer.contains("fs::write("));
    }

    #[test]
    fn fresh_cache_read_is_missing_without_creating_a_bundle_journal() {
        let root = test_directory("fresh-cache-read");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);

        assert!(matches!(
            store
                .read_source_cubemap_zcube(&request, uri())
                .expect("a fresh cache must not require a bundle journal"),
            IblSourceCubemapStagingRead::Missing
        ));
        assert!(
            fs::read_dir(store.bundle_journal_directory())
                .expect("a source read miss must prepare an empty owner journal")
                .next()
                .is_none(),
            "a source read miss must not create a bundle transaction"
        );
    }

    #[test]
    fn environment_read_recovers_a_bundle_staged_after_initial_recovery_before_missing() {
        let root = test_directory("source-miss-after-recovery");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let publisher = store.clone();
        let mut staged = false;

        let result = store.read_source_cubemap_environment_with_snapshot_hooks(
            &request,
            uri(),
            || {
                if staged {
                    return Ok(());
                }
                staged = true;
                publisher
                    .write_source_cubemap_staged_bundle_for_test(
                        &request,
                        uri(),
                        &cubemap([0.25, 0.5, 0.75, 1.0]),
                        None,
                        TransactionFault::CrashAfterStaging(0),
                    )
                    .expect_err("fixture must retain the staged bundle journal");
                Ok(())
            },
            || Ok(()),
            || Ok(()),
            || Ok(()),
        );

        assert!(
            staged,
            "fixture must stage after the initial recovery check"
        );
        assert!(matches!(
            result,
            Err(super::IblSourceCubemapStagingError::MissingSourceCubemap)
        ));
        assert_eq!(
            fs::read_dir(store.bundle_journal_directory())
                .expect("the next reader iteration must recover the staged journal")
                .count(),
            0,
            "a miss must not bypass recovery for a bundle staged after its initial check"
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn environment_read_miss_holds_the_bundle_owner_barrier() {
        let root = test_directory("environment-miss-owner-barrier");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let publisher = store.clone();
        let mut publisher_blocked = false;

        let result = store.read_source_cubemap_environment_with_snapshot_hooks(
            &request,
            uri(),
            || Ok(()),
            || Ok(()),
            || Ok(()),
            || {
                publisher_blocked = true;
                publisher
                    .write_source_cubemap_staged_bundle(
                        &request,
                        uri(),
                        &cubemap([0.25, 0.5, 0.75, 1.0]),
                        None,
                    )
                    .expect_err("a publisher cannot start after the miss barrier is held");
                Ok(())
            },
        );

        assert!(
            publisher_blocked,
            "fixture must attempt publication under the barrier"
        );
        assert!(matches!(
            result,
            Err(super::IblSourceCubemapStagingError::MissingSourceCubemap)
        ));
        assert!(
            !store.source_cubemap_path(&request).exists(),
            "blocked publication must not create a source target"
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn source_read_recovers_a_bundle_staged_after_initial_recovery_before_missing() {
        let root = test_directory("source-only-miss-after-recovery");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let publisher = store.clone();
        let mut staged = false;

        let result = store.read_source_cubemap_zcube_with_snapshot_hooks(
            &request,
            || {
                if staged {
                    return Ok(());
                }
                staged = true;
                publisher
                    .write_source_cubemap_staged_bundle_for_test(
                        &request,
                        uri(),
                        &cubemap([0.25, 0.5, 0.75, 1.0]),
                        None,
                        TransactionFault::CrashAfterStaging(0),
                    )
                    .expect_err("fixture must retain the staged bundle journal");
                Ok(())
            },
            || Ok(()),
        );

        assert!(
            staged,
            "fixture must stage after the initial recovery check"
        );
        assert!(matches!(result, Ok(IblSourceCubemapStagingRead::Missing)));
        assert_eq!(
            fs::read_dir(store.bundle_journal_directory())
                .expect("the next reader iteration must recover the staged journal")
                .count(),
            0,
            "a source-only miss must not bypass recovery for a staged bundle"
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn source_read_miss_holds_the_bundle_owner_barrier() {
        let root = test_directory("source-only-miss-owner-barrier");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let publisher = store.clone();
        let mut publisher_blocked = false;

        let result = store.read_source_cubemap_zcube_with_snapshot_hooks(
            &request,
            || Ok(()),
            || {
                publisher_blocked = true;
                publisher
                    .write_source_cubemap_staged_bundle(
                        &request,
                        uri(),
                        &cubemap([0.25, 0.5, 0.75, 1.0]),
                        None,
                    )
                    .expect_err("a publisher cannot start after the miss barrier is held");
                Ok(())
            },
        );

        assert!(
            publisher_blocked,
            "fixture must attempt publication under the barrier"
        );
        assert!(matches!(result, Ok(IblSourceCubemapStagingRead::Missing)));
        assert!(
            !store.source_cubemap_path(&request).exists(),
            "blocked publication must not create a source target"
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn interrupted_bundle_staging_recovers_before_the_next_read() {
        let root = test_directory("interrupted-staging");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);

        store
            .write_source_cubemap_staged_bundle_for_test(
                &request,
                uri(),
                &cubemap([0.25, 0.5, 0.75, 1.0]),
                None,
                TransactionFault::CrashAfterStaging(0),
            )
            .expect_err("the injected interruption must retain staging recovery evidence");
        assert!(
            store.bundle_journal_directory().exists(),
            "the interrupted bundle must retain its journal"
        );

        assert!(matches!(
            store
                .read_source_cubemap_zcube(&request, uri())
                .expect("the next read must recover the pre-commit bundle"),
            IblSourceCubemapStagingRead::Missing
        ));
        assert_eq!(
            fs::read_dir(store.bundle_journal_directory())
                .expect("recovered journal directory must remain inspectable")
                .count(),
            0,
            "recovery must remove the interrupted journal and staged files"
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn interrupted_bundle_publication_recovers_the_previous_source_and_derived_generation() {
        let root = test_directory("interrupted-bundle");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let initial = cubemap([0.25, 0.5, 0.75, 1.0]);
        let replacement = cubemap([0.75, 0.5, 0.25, 1.0]);

        store
            .write_source_cubemap_staged_bundle(&request, uri(), &initial, None)
            .expect("initial source bundle must publish");
        let source_path = store.source_cubemap_path(&request);
        let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);
        let initial_source_bytes = fs::read(&source_path).expect("initial source must exist");
        let initial_derived_bytes =
            fs::read(&asset_derived_path).expect("initial derived artifact must exist");

        store
            .write_source_cubemap_staged_bundle_for_test(
                &request,
                uri(),
                &replacement,
                None,
                TransactionFault::CrashAfterCommit(0),
            )
            .expect_err("the injected interruption must retain recovery evidence");
        assert_ne!(
            fs::read(&source_path).expect("source first target must have changed"),
            initial_source_bytes
        );
        assert_eq!(
            fs::read(&asset_derived_path).expect("derived second target must remain old"),
            initial_derived_bytes
        );

        assert!(matches!(
            store
                .read_source_cubemap_zcube(&request, uri())
                .expect("the next read must recover the interrupted bundle"),
            IblSourceCubemapStagingRead::Hit(_)
        ));
        assert_eq!(
            fs::read(&source_path).expect("recovered source must exist"),
            initial_source_bytes
        );
        assert_eq!(
            fs::read(&asset_derived_path).expect("recovered derived artifact must exist"),
            initial_derived_bytes
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn environment_read_retries_when_bundle_publication_interleaves_the_snapshot() {
        let root = test_directory("interleaved-read");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let initial = cubemap([0.25, 0.5, 0.75, 1.0]);
        let replacement = cubemap([0.75, 0.5, 0.25, 1.0]);
        store
            .write_source_cubemap_staged_bundle(&request, uri(), &initial, None)
            .expect("initial source bundle must publish");

        let publisher = store.clone();
        let mut published = false;
        let environment = store
            .read_source_cubemap_environment_with_snapshot_hooks(
                &request,
                uri(),
                || Ok(()),
                || {
                    if published {
                        return Ok(());
                    }
                    published = true;
                    publisher
                        .write_source_cubemap_staged_bundle(&request, uri(), &replacement, None)
                        .map(|_| ())
                },
                || Ok(()),
                || Ok(()),
            )
            .expect("the reader must retry after an interleaved bundle publication");

        assert!(
            published,
            "the test hook must publish the replacement bundle"
        );
        assert_eq!(
            environment.mip_chain.source_texels(),
            replacement.source_texels(),
            "the completed read must not combine the initial source with the replacement derived artifact"
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn environment_read_retries_a_missing_recipe_after_reused_source_publication() {
        let root = test_directory("interleaved-reused-source-read");
        let store = IblSourceCubemapStagingStore::new(&root);
        let source_request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let reused_recipe = request(7, [11; 4], 4, 3, 1, 1, IblBakeArtifactContents::PMREM_SH9);
        let source = cubemap([0.25, 0.5, 0.75, 1.0]);
        let reused_source = cubemap_with_pmrem([0.25, 0.5, 0.75, 1.0], 1, 1);
        store
            .write_source_cubemap_staged_bundle(&source_request, uri(), &source, None)
            .expect("initial source bundle must publish");

        let publisher = store.clone();
        let mut published = false;
        let environment = store
            .read_source_cubemap_environment_with_snapshot_hooks(
                &reused_recipe,
                uri(),
                || Ok(()),
                || Ok(()),
                || {
                    if published {
                        return Ok(());
                    }
                    published = true;
                    publisher
                        .asset_derived_store()
                        .write_source_cubemap_asset_derived_artifact(
                            &reused_recipe,
                            &reused_source,
                            None,
                        )
                        .map(|_| ())
                        .map_err(IblSourceCubemapStagingError::AssetDerived)
                },
                || Ok(()),
            )
            .expect("the reader must retry after the missing recipe is published");

        assert!(published, "the test hook must publish the missing recipe");
        assert_eq!(environment.mip_chain.pmrem_face_size(), 1);
        assert_eq!(environment.mip_chain.pmrem_mip_count(), 1);
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn environment_read_retries_a_rejected_derived_artifact_replaced_by_bundle() {
        let root = test_directory("interleaved-rejected-derived-read");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let source = cubemap([0.25, 0.5, 0.75, 1.0]);
        store
            .write_source_cubemap_staged_bundle(&request, uri(), &source, None)
            .expect("initial source bundle must publish");
        let derived_path = store.asset_derived_store().asset_derived_path(&request);
        fs::write(&derived_path, b"rejected derived artifact")
            .expect("fixture must replace the derived artifact with invalid bytes");

        let publisher = store.clone();
        let mut published = false;
        let environment = store
            .read_source_cubemap_environment_with_snapshot_hooks(
                &request,
                uri(),
                || Ok(()),
                || Ok(()),
                || {
                    if published {
                        return Ok(());
                    }
                    published = true;
                    publisher
                        .write_source_cubemap_staged_bundle(&request, uri(), &source, None)
                        .map(|_| ())
                },
                || Ok(()),
            )
            .expect("the reader must retry after a rejected artifact is repaired as a bundle");

        assert!(published, "the test hook must publish the repaired bundle");
        assert_eq!(
            environment.mip_chain.source_texels(),
            source.source_texels()
        );
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    #[test]
    fn standalone_derived_blob_cannot_replace_a_paired_source_bundle() {
        let root = test_directory("reject-standalone-derived");
        let store = IblSourceCubemapStagingStore::new(&root);
        let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
        let source = cubemap([0.25, 0.5, 0.75, 1.0]);
        store
            .write_source_cubemap_staged_bundle(&request, uri(), &source, None)
            .expect("initial source bundle must publish");
        let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);
        let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &source, None)
            .expect("matching standalone fixture must encode");

        assert!(matches!(
            store
                .asset_derived_store()
                .write_asset_derived_blob(&IblBakeArtifactBlob::from_payload(payload)),
            Err(IblBakeArtifactAssetDerivedError::PairedSourceRequiresBundle { .. })
        ));
        fs::remove_dir_all(root).expect("test cache root must be removable");
    }

    fn request(
        revision: u64,
        source_hash: [u32; 4],
        source_face_size: u32,
        source_mip_count: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
        contents: IblBakeArtifactContents,
    ) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            IblBakeKey::source_cubemap(revision, source_hash),
            source_face_size,
            source_mip_count,
        )
        .with_pmrem_layout(pmrem_face_size, pmrem_mip_count)
        .with_required_contents(contents)
    }

    fn cubemap(color: [Real; 4]) -> SourceCubemapMipChain {
        cubemap_with_pmrem(color, 2, 2)
    }

    fn cubemap_with_pmrem(
        color: [Real; 4],
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
    ) -> SourceCubemapMipChain {
        SourceCubemapMipChain::from_equirect_with_pmrem_layout(
            4,
            pmrem_face_size,
            pmrem_mip_count,
            SourceCubemapPrefilterQuality::Fast,
            move |_, _| color,
        )
    }

    fn uri() -> AssetUri {
        AssetUri::parse("res://environment/interrupted-bundle.hdr")
            .expect("test source URI must be valid")
    }

    fn test_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "zircon-ibl-source-staging-{label}-{}-{}",
            std::process::id(),
            TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
