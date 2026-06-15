use crate::core::framework::net::ZrChunkEntry;

use super::{
    writer::header_size, ZrPackAssetEntry, ZrPackDocumentManifest, ZrPackError,
    ZRPACK_FORMAT_VERSION, ZRPACK_MAGIC,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackReader {
    bytes: Vec<u8>,
    manifest: ZrPackDocumentManifest,
}

impl ZrPackReader {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, ZrPackError> {
        let bytes = bytes.into();
        let manifest = read_manifest(&bytes)?;
        validate_manifest_chunks(&bytes, &manifest)?;
        Ok(Self { bytes, manifest })
    }

    pub fn manifest(&self) -> &ZrPackDocumentManifest {
        &self.manifest
    }

    pub fn read_asset(&self, path: &str) -> Result<Vec<u8>, ZrPackError> {
        let asset = self
            .manifest
            .asset(path)
            .ok_or_else(|| ZrPackError::AssetNotFound(path.to_string()))?;
        let chunk = self
            .manifest
            .pack
            .chunks
            .iter()
            .find(|chunk| chunk.hash == asset.chunk_hash)
            .ok_or_else(|| ZrPackError::MissingChunk(path.to_string()))?;
        read_chunk_bytes(&self.bytes, path, asset, chunk)
    }

    pub(crate) fn read_chunk_by_hash(
        &self,
        hash: [u8; 32],
        path_hint: &str,
    ) -> Result<Vec<u8>, ZrPackError> {
        let chunk = self
            .manifest
            .pack
            .chunks
            .iter()
            .find(|chunk| chunk.hash == hash)
            .ok_or_else(|| ZrPackError::MissingChunk(path_hint.to_string()))?;
        read_chunk_range_bytes(&self.bytes, path_hint, chunk)
    }
}

fn read_manifest(bytes: &[u8]) -> Result<ZrPackDocumentManifest, ZrPackError> {
    if bytes.len() < header_size() {
        return Err(ZrPackError::HeaderTooSmall);
    }
    if bytes[0..4] != ZRPACK_MAGIC {
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

fn validate_manifest_chunks(
    bytes: &[u8],
    manifest: &ZrPackDocumentManifest,
) -> Result<(), ZrPackError> {
    for asset in &manifest.assets {
        let chunk = manifest
            .pack
            .chunks
            .iter()
            .find(|chunk| chunk.hash == asset.chunk_hash)
            .ok_or_else(|| ZrPackError::MissingChunk(asset.path.clone()))?;
        let _ = read_chunk_bytes(bytes, &asset.path, asset, chunk)?;
    }
    Ok(())
}

fn read_chunk_bytes(
    bytes: &[u8],
    path: &str,
    asset: &ZrPackAssetEntry,
    chunk: &ZrChunkEntry,
) -> Result<Vec<u8>, ZrPackError> {
    if u64::from(chunk.size) != asset.size {
        return Err(ZrPackError::ChunkOutOfBounds(path.to_string()));
    }
    read_chunk_range_bytes(bytes, path, chunk)
}

fn read_chunk_range_bytes(
    bytes: &[u8],
    path: &str,
    chunk: &ZrChunkEntry,
) -> Result<Vec<u8>, ZrPackError> {
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
