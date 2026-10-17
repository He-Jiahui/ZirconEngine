//! Portable assets retain the cache's positional DTO; HTTP supplies transfer compression.
use super::cache_payload::{from_imported_for_write, ArtifactCacheAsset};
use crate::asset::{AssetId, ImportedAsset};
use crate::core::resource::ResourceRecord;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use zircon_runtime_interface::serialization::{
    PositionalDecodeError, PositionalEncodeError, PositionalEncoding,
};
const RAW_MAGIC: &[u8; 8] = b"ZRRAW001";
pub const RAW_ASSET_MAX_BYTES: u64 = 256 * 1024 * 1024;
pub const RAW_CATALOG_MAX_BYTES: u64 = 16 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawAssetCatalog {
    pub schema_version: u32,
    pub target_platform: String,
    pub assets: Vec<RawAssetCatalogEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<RawSceneCatalogEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub startup_assets: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_roots: Vec<AssetId>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawAssetCatalogEntry {
    pub record: ResourceRecord,
    pub payload_bytes: u64,
    pub content_hash: [u8; 32],
}
impl RawAssetCatalogEntry {
    pub fn id(&self) -> AssetId {
        self.record.id
    }
    pub fn relative_payload_path(&self) -> String {
        format!(
            "assets/{}.zr-raw",
            blake3::Hash::from_bytes(self.content_hash).to_hex()
        )
    }
}
#[derive(Debug, thiserror::Error)]
pub enum RawAssetError {
    #[error("Raw asset magic is invalid")]
    Magic,
    #[error("Raw asset has {actual} bytes, exceeding {limit}")]
    ByteLimit { actual: u64, limit: u64 },
    #[error("Raw asset encode failed: {0}")]
    Encode(#[from] PositionalEncodeError),
    #[error("Raw asset decode failed: {0}")]
    Decode(#[from] PositionalDecodeError),
    #[error("Raw asset payload is invalid: {0}")]
    Payload(String),
    #[error("Raw asset I/O failed: {0}")]
    Io(#[from] io::Error),
}
/// The caller stages and atomically publishes the resulting immutable content.
/// The model view borrows its large buffers all the way through this writer.
pub fn write_raw_asset(
    mut writer: impl Write,
    asset: &ImportedAsset,
    limit: u64,
) -> Result<(), RawAssetError> {
    if limit < RAW_MAGIC.len() as u64 {
        return Err(RawAssetError::ByteLimit {
            actual: RAW_MAGIC.len() as u64,
            limit,
        });
    }
    let view = from_imported_for_write(asset)
        .map_err(|error| RawAssetError::Payload(error.to_string()))?;
    writer.write_all(RAW_MAGIC)?;
    PositionalEncoding::FixedLittleEndian.encode_into(
        &mut writer,
        &view,
        limit - RAW_MAGIC.len() as u64,
    )?;
    Ok(())
}
pub fn read_raw_asset(bytes: &[u8], limit: u64) -> Result<ImportedAsset, RawAssetError> {
    if bytes.len() as u64 > limit {
        return Err(RawAssetError::ByteLimit {
            actual: bytes.len() as u64,
            limit,
        });
    }
    let bytes = bytes.strip_prefix(RAW_MAGIC).ok_or(RawAssetError::Magic)?;
    let dto: ArtifactCacheAsset =
        PositionalEncoding::FixedLittleEndian.decode_bounded(bytes, limit)?;
    dto.into_imported()
        .map_err(|error| RawAssetError::Payload(error.to_string()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawSceneCatalogEntry {
    pub payload_bytes: u64,
    pub content_hash: [u8; 32],
}
impl RawSceneCatalogEntry {
    pub fn relative_payload_path(&self) -> String {
        format!(
            "scenes/{}.json",
            blake3::Hash::from_bytes(self.content_hash).to_hex()
        )
    }
}
