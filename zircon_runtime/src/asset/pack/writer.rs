use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::net::{ZrChunkEntry, ZrPackManifest};

use super::{
    zrpack_content_hash, ZrPackAssetEntry, ZrPackDocumentManifest, ZrPackError,
    ZRPACK_FORMAT_VERSION, ZRPACK_MAGIC,
};

const ZRPACK_HEADER_SIZE: usize = 24;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackInputAsset {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackWriteReport {
    pub manifest: ZrPackDocumentManifest,
    pub bytes: Vec<u8>,
    pub deduplicated_assets: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackWriter;

impl ZrPackInputAsset {
    pub fn new(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

impl ZrPackWriter {
    pub fn write(
        assets: impl IntoIterator<Item = ZrPackInputAsset>,
    ) -> Result<ZrPackWriteReport, ZrPackError> {
        let mut assets = assets.into_iter().collect::<Vec<_>>();
        assets.sort_by(|left, right| left.path.cmp(&right.path));
        reject_duplicate_paths(&assets)?;

        let mut bytes = vec![0; ZRPACK_HEADER_SIZE];
        let mut chunk_offsets = BTreeMap::new();
        let mut chunk_entries = Vec::new();
        let mut asset_entries = Vec::new();
        let mut deduplicated_assets = Vec::new();

        for asset in assets {
            let hash = zrpack_content_hash(&asset.bytes);
            if let Some(offset) = chunk_offsets.get(&hash).copied() {
                deduplicated_assets.push(asset.path.clone());
                asset_entries.push(ZrPackAssetEntry::new(
                    asset.path,
                    hash,
                    asset.bytes.len() as u64,
                ));
                debug_assert!(offset <= bytes.len() as u64);
                continue;
            }

            let offset = u64::try_from(bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
            let size = u32::try_from(asset.bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
            bytes.extend_from_slice(&asset.bytes);
            chunk_offsets.insert(hash, offset);
            chunk_entries.push(ZrChunkEntry::new(hash, offset, size));
            asset_entries.push(ZrPackAssetEntry::new(asset.path, hash, u64::from(size)));
        }

        chunk_entries.sort_by(|left, right| left.hash.cmp(&right.hash));
        let total_size = chunk_entries
            .iter()
            .map(|chunk| u64::from(chunk.size))
            .sum();
        let manifest = ZrPackDocumentManifest::new(
            ZrPackManifest {
                version: ZRPACK_FORMAT_VERSION,
                chunks: chunk_entries,
                total_size,
            },
            asset_entries,
        );
        let manifest_bytes = serde_json::to_vec(&manifest)
            .map_err(|error| ZrPackError::ManifestDecode(error.to_string()))?;
        let manifest_offset = u64::try_from(bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        let manifest_size =
            u64::try_from(manifest_bytes.len()).map_err(|_| ZrPackError::SizeOverflow)?;
        bytes.extend_from_slice(&manifest_bytes);
        write_header(
            &mut bytes[..ZRPACK_HEADER_SIZE],
            manifest_offset,
            manifest_size,
        );

        Ok(ZrPackWriteReport {
            manifest,
            bytes,
            deduplicated_assets,
        })
    }
}

fn reject_duplicate_paths(assets: &[ZrPackInputAsset]) -> Result<(), ZrPackError> {
    let mut paths = BTreeSet::new();
    for asset in assets {
        if !paths.insert(asset.path.clone()) {
            return Err(ZrPackError::DuplicateAssetPath(asset.path.clone()));
        }
    }
    Ok(())
}

fn write_header(header: &mut [u8], manifest_offset: u64, manifest_size: u64) {
    header[0..4].copy_from_slice(&ZRPACK_MAGIC);
    header[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    header[16..24].copy_from_slice(&manifest_size.to_le_bytes());
}

pub(super) fn header_size() -> usize {
    ZRPACK_HEADER_SIZE
}
