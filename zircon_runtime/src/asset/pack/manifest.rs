use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::core::framework::net::ZrPackManifest;

pub const ZRPACK_MAGIC: [u8; 4] = *b"ZRPK";
pub const ZRPACK_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackDocumentManifest {
    pub pack: ZrPackManifest,
    pub assets: Vec<ZrPackAssetEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackAssetEntry {
    pub path: String,
    pub chunk_hash: [u8; 32],
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZrPackError {
    InvalidMagic,
    UnsupportedVersion(u32),
    HeaderTooSmall,
    ManifestOutOfBounds,
    ManifestTrailingBytes,
    ManifestDecode(String),
    UnsafeAssetPath(String),
    UnnormalizedAssetPath { path: String, normalized: String },
    UnsortedAssetPaths,
    DuplicateAssetPath(String),
    DuplicateChunkHash,
    UnsortedChunkHashes,
    PackTotalSizeMismatch,
    PackChunkTableMismatch,
    PayloadExtentMismatch,
    DeltaRemovedAssetsMismatch,
    DeltaChangedAssetsMismatch,
    DeltaChunkTableMismatch,
    MissingChunk(String),
    ChunkOutOfBounds(String),
    ChunkHashMismatch(String),
    AssetNotFound(String),
    DeltaBaseManifestMismatch,
    DeltaTargetManifestMismatch,
    SizeOverflow,
}

impl ZrPackDocumentManifest {
    pub fn new(pack: ZrPackManifest, assets: Vec<ZrPackAssetEntry>) -> Self {
        Self { pack, assets }
    }

    pub fn asset(&self, path: &str) -> Option<&ZrPackAssetEntry> {
        self.assets.iter().find(|asset| asset.path == path)
    }
}

impl ZrPackAssetEntry {
    pub fn new(path: impl Into<String>, chunk_hash: [u8; 32], size: u64) -> Self {
        Self {
            path: path.into(),
            chunk_hash,
            size,
        }
    }
}

impl fmt::Display for ZrPackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "zrpack header magic is invalid"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "zrpack format version {version} is unsupported")
            }
            Self::HeaderTooSmall => write!(formatter, "zrpack header is too small"),
            Self::ManifestOutOfBounds => {
                write!(formatter, "zrpack manifest range is out of bounds")
            }
            Self::ManifestTrailingBytes => {
                write!(
                    formatter,
                    "zrpack manifest must end at the artifact boundary"
                )
            }
            Self::ManifestDecode(error) => {
                write!(formatter, "failed to decode zrpack manifest: {error}")
            }
            Self::UnsafeAssetPath(path) => {
                write!(
                    formatter,
                    "zrpack asset path {path} must be a safe relative asset path"
                )
            }
            Self::UnnormalizedAssetPath { path, normalized } => {
                write!(
                    formatter,
                    "zrpack asset path {path} must use normalized relative asset path {normalized}"
                )
            }
            Self::UnsortedAssetPaths => {
                write!(formatter, "zrpack asset paths must be sorted by asset path")
            }
            Self::DuplicateAssetPath(path) => {
                write!(formatter, "zrpack asset path {path} is duplicated")
            }
            Self::DuplicateChunkHash => {
                write!(formatter, "zrpack chunk hash is duplicated")
            }
            Self::UnsortedChunkHashes => {
                write!(
                    formatter,
                    "zrpack chunk hashes must be sorted by chunk hash"
                )
            }
            Self::PackTotalSizeMismatch => {
                write!(
                    formatter,
                    "zrpack pack total size does not match chunk table"
                )
            }
            Self::PackChunkTableMismatch => {
                write!(
                    formatter,
                    "zrpack chunk table does not match manifest assets"
                )
            }
            Self::PayloadExtentMismatch => {
                write!(
                    formatter,
                    "zrpack payload extent does not match manifest offset"
                )
            }
            Self::DeltaRemovedAssetsMismatch => {
                write!(
                    formatter,
                    "zrpack delta removed assets do not match base and target manifests"
                )
            }
            Self::DeltaChangedAssetsMismatch => {
                write!(
                    formatter,
                    "zrpack delta changed assets do not match base and target manifests"
                )
            }
            Self::DeltaChunkTableMismatch => {
                write!(
                    formatter,
                    "zrpack delta chunk table does not match changed assets"
                )
            }
            Self::MissingChunk(path) => {
                write!(formatter, "zrpack asset {path} references a missing chunk")
            }
            Self::ChunkOutOfBounds(path) => write!(
                formatter,
                "zrpack asset {path} chunk range is out of bounds"
            ),
            Self::ChunkHashMismatch(path) => write!(
                formatter,
                "zrpack asset {path} chunk payload does not match its content hash"
            ),
            Self::AssetNotFound(path) => write!(formatter, "zrpack asset {path} was not found"),
            Self::DeltaBaseManifestMismatch => {
                write!(
                    formatter,
                    "zrpack delta base manifest does not match the installed pack"
                )
            }
            Self::DeltaTargetManifestMismatch => {
                write!(
                    formatter,
                    "zrpack delta target manifest could not be reconstructed"
                )
            }
            Self::SizeOverflow => {
                write!(formatter, "zrpack size does not fit into the binary format")
            }
        }
    }
}

impl std::error::Error for ZrPackError {}

pub(crate) fn validate_zrpack_document_manifest(
    manifest: &ZrPackDocumentManifest,
) -> Result<(), ZrPackError> {
    if manifest.pack.version != ZRPACK_FORMAT_VERSION {
        return Err(ZrPackError::UnsupportedVersion(manifest.pack.version));
    }
    validate_zrpack_asset_entries(&manifest.assets)?;
    validate_zrpack_chunk_table(manifest)
}

pub(crate) fn validate_zrpack_asset_entries(
    assets: &[ZrPackAssetEntry],
) -> Result<(), ZrPackError> {
    let mut seen_paths = BTreeSet::new();
    let mut paths = Vec::with_capacity(assets.len());
    for asset in assets {
        validate_zrpack_asset_path(&asset.path)?;
        if !seen_paths.insert(asset.path.clone()) {
            return Err(ZrPackError::DuplicateAssetPath(asset.path.clone()));
        }
        paths.push(asset.path.clone());
    }
    if paths != sorted_asset_paths(&paths) {
        return Err(ZrPackError::UnsortedAssetPaths);
    }
    Ok(())
}

pub(crate) fn validate_zrpack_asset_path_list(paths: &[String]) -> Result<(), ZrPackError> {
    let mut seen_paths = BTreeSet::new();
    for path in paths {
        validate_zrpack_asset_path(path)?;
        if !seen_paths.insert(path.clone()) {
            return Err(ZrPackError::DuplicateAssetPath(path.clone()));
        }
    }
    if paths != sorted_asset_paths(paths) {
        return Err(ZrPackError::UnsortedAssetPaths);
    }
    Ok(())
}

pub(crate) fn validate_zrpack_asset_path(path: &str) -> Result<(), ZrPackError> {
    let normalized = normalized_zrpack_asset_path(path);
    if !is_safe_normalized_zrpack_asset_path(&normalized) {
        return Err(ZrPackError::UnsafeAssetPath(path.to_string()));
    }
    if normalized != path {
        return Err(ZrPackError::UnnormalizedAssetPath {
            path: path.to_string(),
            normalized,
        });
    }
    Ok(())
}

fn validate_zrpack_chunk_table(manifest: &ZrPackDocumentManifest) -> Result<(), ZrPackError> {
    let chunk_hashes = manifest
        .pack
        .chunks
        .iter()
        .map(|chunk| chunk.hash)
        .collect::<Vec<_>>();
    if chunk_hashes.iter().collect::<BTreeSet<_>>().len() != chunk_hashes.len() {
        return Err(ZrPackError::DuplicateChunkHash);
    }
    if chunk_hashes != sorted_chunk_hashes(&chunk_hashes) {
        return Err(ZrPackError::UnsortedChunkHashes);
    }

    let total_size = manifest
        .pack
        .chunks
        .iter()
        .try_fold(0_u64, |total, chunk| {
            total
                .checked_add(u64::from(chunk.size))
                .ok_or(ZrPackError::SizeOverflow)
        })?;
    if manifest.pack.total_size != total_size {
        return Err(ZrPackError::PackTotalSizeMismatch);
    }

    let chunk_sizes = manifest
        .pack
        .chunks
        .iter()
        .map(|chunk| (chunk.hash, u64::from(chunk.size)))
        .collect::<BTreeMap<_, _>>();
    let mut asset_hashes = BTreeSet::new();
    for asset in &manifest.assets {
        let chunk_size = chunk_sizes
            .get(&asset.chunk_hash)
            .ok_or_else(|| ZrPackError::MissingChunk(asset.path.clone()))?;
        if asset.size != *chunk_size {
            return Err(ZrPackError::ChunkOutOfBounds(asset.path.clone()));
        }
        asset_hashes.insert(asset.chunk_hash);
    }

    if asset_hashes != chunk_sizes.keys().copied().collect::<BTreeSet<_>>() {
        return Err(ZrPackError::PackChunkTableMismatch);
    }

    Ok(())
}

fn normalized_zrpack_asset_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

fn is_safe_normalized_zrpack_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(':')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn sorted_asset_paths(paths: &[String]) -> Vec<String> {
    let mut sorted = paths.to_vec();
    sorted.sort();
    sorted
}

fn sorted_chunk_hashes(hashes: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let mut sorted = hashes.to_vec();
    sorted.sort();
    sorted
}
