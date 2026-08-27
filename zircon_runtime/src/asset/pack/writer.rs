use std::borrow::Borrow;
use std::collections::BTreeMap;

use super::manifest::validate_zrpack_asset_path;
use super::{
    zrpack_content_hash, ZrChunkEntry, ZrPackAssetEntry, ZrPackDocumentManifest, ZrPackError,
    ZrPackManifest, ZRPACK_FORMAT_VERSION, ZRPACK_MAGIC,
};

const ZRPACK_HEADER_SIZE: usize = 24;

#[derive(Debug, PartialEq, Eq)]
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
    pub fn write<I, A>(assets: I) -> Result<ZrPackWriteReport, ZrPackError>
    where
        I: IntoIterator<Item = A>,
        A: Borrow<ZrPackInputAsset>,
    {
        let mut assets = assets.into_iter().collect::<Vec<_>>();
        validate_asset_paths(&assets)?;
        sort_assets_by_path(&mut assets);
        reject_duplicate_paths(&assets)?;

        let mut bytes = vec![0; ZRPACK_HEADER_SIZE];
        let mut chunk_offsets = BTreeMap::new();
        let mut chunk_entries = Vec::with_capacity(assets.len());
        let mut asset_entries = Vec::with_capacity(assets.len());
        let mut deduplicated_assets = Vec::with_capacity(assets.len());

        for asset in &assets {
            let asset = input_asset(asset);
            let hash = zrpack_content_hash(&asset.bytes);
            if let Some(offset) = chunk_offsets.get(&hash).copied() {
                deduplicated_assets.push(asset.path.clone());
                asset_entries.push(ZrPackAssetEntry::new(
                    asset.path.clone(),
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
            asset_entries.push(ZrPackAssetEntry::new(
                asset.path.clone(),
                hash,
                u64::from(size),
            ));
        }

        chunk_entries.sort_unstable_by(|left, right| left.hash.cmp(&right.hash));
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

fn input_asset<A>(asset: &A) -> &ZrPackInputAsset
where
    A: Borrow<ZrPackInputAsset>,
{
    asset.borrow()
}

fn sort_assets_by_path<A>(assets: &mut [A])
where
    A: Borrow<ZrPackInputAsset>,
{
    assets.sort_unstable_by(|left, right| input_asset(left).path.cmp(&input_asset(right).path));
}

fn validate_asset_paths<A>(assets: &[A]) -> Result<(), ZrPackError>
where
    A: Borrow<ZrPackInputAsset>,
{
    for asset in assets {
        let asset = input_asset(asset);
        validate_zrpack_asset_path(&asset.path)?;
    }
    Ok(())
}

fn reject_duplicate_paths<A>(assets: &[A]) -> Result<(), ZrPackError>
where
    A: Borrow<ZrPackInputAsset>,
{
    if let Some(pair) = assets
        .windows(2)
        .find(|pair| input_asset(&pair[0]).path == input_asset(&pair[1]).path)
    {
        return Err(ZrPackError::DuplicateAssetPath(
            input_asset(&pair[1]).path.clone(),
        ));
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

#[cfg(test)]
#[path = "writer/optimization_tests.rs"]
mod optimization_tests;
