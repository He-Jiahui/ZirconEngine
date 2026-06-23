use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::asset::pack::{
    zrpack_content_hash, ZrPackAssetEntry, ZrPackDeltaDocumentManifest, ZrPackDeltaInstallError,
    ZrPackDeltaInstaller, ZrPackDeltaReader, ZrPackDeltaWriter, ZrPackDocumentManifest,
    ZrPackError, ZrPackInputAsset, ZrPackPromotionMethod, ZrPackReader, ZrPackTrimConfig,
    ZrPackTrimInputAsset, ZrPackTrimPlanner, ZrPackTrimReason, ZrPackWriter, ZRPACK_DELTA_MAGIC,
    ZRPACK_FORMAT_VERSION, ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION, ZRPACK_MAGIC,
};
use crate::core::framework::net::{ZrChunkEntry, ZrPackManifest};

mod basic;
mod delta_installer;
mod delta_pack;
mod delta_reader_validation;
mod reader_validation;
mod trim;

fn pack_asset_entry(path: impl Into<String>) -> ZrPackAssetEntry {
    pack_asset_entry_with_payload(path, b"data")
}

fn pack_asset_entry_with_payload(path: impl Into<String>, payload: &[u8]) -> ZrPackAssetEntry {
    ZrPackAssetEntry::new(
        path,
        zrpack_content_hash(payload),
        u64::try_from(payload.len()).unwrap(),
    )
}

fn chunks_for_asset_entries(assets: &[ZrPackAssetEntry]) -> Vec<ZrChunkEntry> {
    let mut unique_chunks = std::collections::BTreeMap::new();
    for asset in assets {
        unique_chunks
            .entry(asset.chunk_hash)
            .or_insert_with(|| u32::try_from(asset.size).unwrap());
    }
    let mut offset = ZRPACK_TEST_HEADER_SIZE as u64;
    unique_chunks
        .into_iter()
        .map(|(hash, size)| {
            let entry = ZrChunkEntry::new(hash, offset, size);
            offset += u64::from(size);
            entry
        })
        .collect()
}

fn total_chunk_size(chunks: &[ZrChunkEntry]) -> u64 {
    chunks.iter().map(|chunk| u64::from(chunk.size)).sum()
}

fn payload_bytes_for_chunks(chunks: &[ZrChunkEntry]) -> Vec<u8> {
    vec![0; usize::try_from(total_chunk_size(chunks)).unwrap()]
}

fn malformed_pack_bytes_with_assets(assets: Vec<ZrPackAssetEntry>) -> Vec<u8> {
    malformed_pack_bytes(pack_document_manifest_with_assets(assets))
}

fn malformed_pack_bytes(manifest: ZrPackDocumentManifest) -> Vec<u8> {
    let payload = payload_bytes_for_chunks(&manifest.pack.chunks);
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let manifest_offset = (ZRPACK_TEST_HEADER_SIZE + payload.len()) as u64;
    let manifest_size = manifest_bytes.len() as u64;
    let mut bytes = vec![0; ZRPACK_TEST_HEADER_SIZE];
    bytes.extend_from_slice(&payload);
    bytes.extend_from_slice(&manifest_bytes);
    bytes[0..4].copy_from_slice(&ZRPACK_MAGIC);
    bytes[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    bytes[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    bytes[16..24].copy_from_slice(&manifest_size.to_le_bytes());
    bytes
}

fn delta_manifest_with_assets(
    base_assets: Vec<ZrPackAssetEntry>,
    target_assets: Vec<ZrPackAssetEntry>,
    changed_assets: Vec<ZrPackAssetEntry>,
    removed_assets: Vec<String>,
) -> ZrPackDeltaDocumentManifest {
    let chunks = chunks_for_asset_entries(&changed_assets);
    ZrPackDeltaDocumentManifest {
        format_version: ZRPACK_FORMAT_VERSION,
        base: pack_document_manifest_with_assets(base_assets),
        target: pack_document_manifest_with_assets(target_assets),
        chunks,
        changed_assets,
        removed_assets,
    }
}

fn malformed_delta_bytes(manifest: ZrPackDeltaDocumentManifest) -> Vec<u8> {
    let payload = payload_bytes_for_chunks(&manifest.chunks);
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let manifest_offset = (ZRPACK_TEST_HEADER_SIZE + payload.len()) as u64;
    let manifest_size = manifest_bytes.len() as u64;
    let mut bytes = vec![0; ZRPACK_TEST_HEADER_SIZE];
    bytes.extend_from_slice(&payload);
    bytes.extend_from_slice(&manifest_bytes);
    bytes[0..4].copy_from_slice(&ZRPACK_DELTA_MAGIC);
    bytes[4..8].copy_from_slice(&ZRPACK_FORMAT_VERSION.to_le_bytes());
    bytes[8..16].copy_from_slice(&manifest_offset.to_le_bytes());
    bytes[16..24].copy_from_slice(&manifest_size.to_le_bytes());
    bytes
}

fn bytes_with_manifest_gap(mut bytes: Vec<u8>, gap: &[u8]) -> Vec<u8> {
    let manifest_offset =
        usize::try_from(u64::from_le_bytes(bytes[8..16].try_into().unwrap())).unwrap();
    bytes.splice(manifest_offset..manifest_offset, gap.iter().copied());
    let new_manifest_offset = u64::try_from(manifest_offset + gap.len()).unwrap();
    bytes[8..16].copy_from_slice(&new_manifest_offset.to_le_bytes());
    bytes
}

fn bytes_with_manifest_trailing_bytes(mut bytes: Vec<u8>, trailing_bytes: &[u8]) -> Vec<u8> {
    bytes.extend_from_slice(trailing_bytes);
    bytes
}

fn pack_document_manifest_with_assets(assets: Vec<ZrPackAssetEntry>) -> ZrPackDocumentManifest {
    let chunks = chunks_for_asset_entries(&assets);
    let total_size = total_chunk_size(&chunks);
    ZrPackDocumentManifest::new(
        ZrPackManifest {
            version: ZRPACK_FORMAT_VERSION,
            chunks,
            total_size,
        },
        assets,
    )
}

const ZRPACK_TEST_HEADER_SIZE: usize = 24;

fn unique_pack_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "zircon-pack-{label}-{}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
