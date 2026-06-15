use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::net::ZrChunkEntry;

use super::{
    writer::{header_size, ZrPackInputAsset, ZrPackWriteReport, ZrPackWriter},
    ZrPackAssetEntry, ZrPackDocumentManifest, ZrPackError, ZrPackReader, ZRPACK_FORMAT_VERSION,
};

pub const ZRPACK_DELTA_MAGIC: [u8; 4] = *b"ZRPD";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackDeltaDocumentManifest {
    pub format_version: u32,
    pub base: ZrPackDocumentManifest,
    pub target: ZrPackDocumentManifest,
    pub chunks: Vec<ZrChunkEntry>,
    pub changed_assets: Vec<ZrPackAssetEntry>,
    pub removed_assets: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackDeltaWriteReport {
    pub manifest: ZrPackDeltaDocumentManifest,
    pub bytes: Vec<u8>,
    pub changed_assets: Vec<String>,
    pub removed_assets: Vec<String>,
    pub reused_assets: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackDeltaWriter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackDeltaReader {
    bytes: Vec<u8>,
    manifest: ZrPackDeltaDocumentManifest,
}

impl ZrPackDeltaDocumentManifest {
    pub fn changed_asset(&self, path: &str) -> Option<&ZrPackAssetEntry> {
        self.changed_assets.iter().find(|asset| asset.path == path)
    }
}

impl ZrPackDeltaWriter {
    pub fn write(
        base: &ZrPackReader,
        target: &ZrPackReader,
    ) -> Result<ZrPackDeltaWriteReport, ZrPackError> {
        let base_hashes = base
            .manifest()
            .pack
            .chunks
            .iter()
            .map(|chunk| chunk.hash)
            .collect::<BTreeSet<_>>();
        let target_paths = target
            .manifest()
            .assets
            .iter()
            .map(|asset| asset.path.clone())
            .collect::<BTreeSet<_>>();

        let mut removed_assets = base
            .manifest()
            .assets
            .iter()
            .filter(|asset| !target_paths.contains(&asset.path))
            .map(|asset| asset.path.clone())
            .collect::<Vec<_>>();
        removed_assets.sort();

        let mut changed_asset_entries = Vec::new();
        let mut changed_assets = Vec::new();
        let mut reused_assets = Vec::new();
        let mut chunk_source_paths = BTreeMap::new();

        let mut target_assets = target.manifest().assets.clone();
        target_assets.sort_by(|left, right| left.path.cmp(&right.path));
        for asset in target_assets {
            if base_hashes.contains(&asset.chunk_hash) {
                reused_assets.push(asset.path);
                continue;
            }
            chunk_source_paths
                .entry(asset.chunk_hash)
                .or_insert_with(|| asset.path.clone());
            changed_assets.push(asset.path.clone());
            changed_asset_entries.push(asset);
        }

        let mut bytes = vec![0; header_size()];
        let mut chunks = Vec::new();
        for (hash, path) in chunk_source_paths {
            let chunk_bytes = target.read_asset(&path)?;
            let offset = u64::try_from(bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
            let size = u32::try_from(chunk_bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
            bytes.extend_from_slice(&chunk_bytes);
            chunks.push(ZrChunkEntry::new(hash, offset, size));
        }

        let manifest = ZrPackDeltaDocumentManifest {
            format_version: ZRPACK_FORMAT_VERSION,
            base: base.manifest().clone(),
            target: target.manifest().clone(),
            chunks,
            changed_assets: changed_asset_entries,
            removed_assets: removed_assets.clone(),
        };
        let manifest_bytes = serde_json::to_vec(&manifest)
            .map_err(|error| ZrPackError::ManifestDecode(error.to_string()))?;
        let manifest_offset = u64::try_from(bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        let manifest_size =
            u64::try_from(manifest_bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        bytes.extend_from_slice(&manifest_bytes);
        write_delta_header(&mut bytes[..header_size()], manifest_offset, manifest_size);

        Ok(ZrPackDeltaWriteReport {
            manifest,
            bytes,
            changed_assets,
            removed_assets,
            reused_assets,
        })
    }
}

impl ZrPackDeltaReader {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, ZrPackError> {
        let bytes = bytes.into();
        let manifest = read_delta_manifest(&bytes)?;
        validate_delta_chunks(&bytes, &manifest)?;
        Ok(Self { bytes, manifest })
    }

    pub fn manifest(&self) -> &ZrPackDeltaDocumentManifest {
        &self.manifest
    }

    pub fn read_changed_asset(&self, path: &str) -> Result<Vec<u8>, ZrPackError> {
        let asset = self
            .manifest
            .changed_asset(path)
            .ok_or_else(|| ZrPackError::AssetNotFound(path.to_string()))?;
        let chunk = self
            .manifest
            .chunks
            .iter()
            .find(|chunk| chunk.hash == asset.chunk_hash)
            .ok_or_else(|| ZrPackError::MissingChunk(path.to_string()))?;
        read_delta_chunk_bytes(&self.bytes, path, asset, chunk)
    }

    pub fn apply_to_base(&self, base: &ZrPackReader) -> Result<ZrPackWriteReport, ZrPackError> {
        if base.manifest() != &self.manifest.base {
            return Err(ZrPackError::DeltaBaseManifestMismatch);
        }

        let mut assets = Vec::with_capacity(self.manifest.target.assets.len());
        for asset in &self.manifest.target.assets {
            let bytes = if self.manifest.changed_asset(&asset.path).is_some() {
                self.read_changed_asset(&asset.path)?
            } else {
                base.read_chunk_by_hash(asset.chunk_hash, &asset.path)?
            };
            if u64::try_from(bytes.len()).map_err(|_| ZrPackError::SizeOverflow)? != asset.size {
                return Err(ZrPackError::ChunkOutOfBounds(asset.path.clone()));
            }
            assets.push(ZrPackInputAsset::new(asset.path.clone(), bytes));
        }

        let report = ZrPackWriter::write(assets)?;
        if report.manifest != self.manifest.target {
            return Err(ZrPackError::DeltaTargetManifestMismatch);
        }
        Ok(report)
    }
}

fn read_delta_manifest(bytes: &[u8]) -> Result<ZrPackDeltaDocumentManifest, ZrPackError> {
    if bytes.len() < header_size() {
        return Err(ZrPackError::HeaderTooSmall);
    }
    if bytes[0..4] != ZRPACK_DELTA_MAGIC {
        return Err(ZrPackError::InvalidMagic);
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().expect("header version bytes"));
    if version != ZRPACK_FORMAT_VERSION {
        return Err(ZrPackError::UnsupportedVersion(version));
    }
    let manifest_offset =
        u64::from_le_bytes(bytes[8..16].try_into().expect("header offset bytes")) as usize;
    let manifest_size =
        u64::from_le_bytes(bytes[16..24].try_into().expect("header size bytes")) as usize;
    let manifest_end = manifest_offset
        .checked_add(manifest_size)
        .ok_or(ZrPackError::ManifestOutOfBounds)?;
    if manifest_offset < header_size() || manifest_end > bytes.len() {
        return Err(ZrPackError::ManifestOutOfBounds);
    }
    serde_json::from_slice(&bytes[manifest_offset..manifest_end])
        .map_err(|error| ZrPackError::ManifestDecode(error.to_string()))
}

fn validate_delta_chunks(
    bytes: &[u8],
    manifest: &ZrPackDeltaDocumentManifest,
) -> Result<(), ZrPackError> {
    for asset in &manifest.changed_assets {
        let chunk = manifest
            .chunks
            .iter()
            .find(|chunk| chunk.hash == asset.chunk_hash)
            .ok_or_else(|| ZrPackError::MissingChunk(asset.path.clone()))?;
        let _ = read_delta_chunk_bytes(bytes, &asset.path, asset, chunk)?;
    }
    Ok(())
}

fn read_delta_chunk_bytes(
    bytes: &[u8],
    path: &str,
    asset: &ZrPackAssetEntry,
    chunk: &ZrChunkEntry,
) -> Result<Vec<u8>, ZrPackError> {
    if u64::from(chunk.size) != asset.size {
        return Err(ZrPackError::ChunkOutOfBounds(path.to_string()));
    }
    let start = usize::try_from(chunk.offset)
        .map_err(|_| ZrPackError::ChunkOutOfBounds(path.to_string()))?;
    let size =
        usize::try_from(chunk.size).map_err(|_| ZrPackError::ChunkOutOfBounds(path.to_string()))?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| ZrPackError::ChunkOutOfBounds(path.to_string()))?;
    if start < header_size() || end > bytes.len() {
        return Err(ZrPackError::ChunkOutOfBounds(path.to_string()));
    }
    Ok(bytes[start..end].to_vec())
}

fn write_delta_header(header: &mut [u8], manifest_offset: u64, manifest_size: u64) {
    header[0..4].copy_from_slice(&ZRPACK_DELTA_MAGIC);
    header[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    header[16..24].copy_from_slice(&manifest_size.to_le_bytes());
}
