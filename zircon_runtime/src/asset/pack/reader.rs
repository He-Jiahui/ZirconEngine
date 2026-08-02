use super::dedup::zrpack_content_hash;
use super::manifest::validate_zrpack_document_manifest;
use super::{
    ZRPACK_FORMAT_VERSION, ZRPACK_MAGIC, ZrChunkEntry, ZrPackAssetEntry, ZrPackDocumentManifest,
    ZrPackError, writer::header_size,
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
        validate_zrpack_document_manifest(&manifest)?;
        validate_chunk_payload_extent(&bytes, &manifest.pack.chunks)?;
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
            .binary_search_by_key(&asset.chunk_hash, |chunk| chunk.hash)
            .ok()
            .map(|index| &self.manifest.pack.chunks[index])
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
            .binary_search_by_key(&hash, |chunk| chunk.hash)
            .ok()
            .map(|index| &self.manifest.pack.chunks[index])
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
    let version = read_header_u32(bytes, 4)?;
    if version != ZRPACK_FORMAT_VERSION {
        return Err(ZrPackError::UnsupportedVersion(version));
    }
    let manifest_offset = read_header_u64(bytes, 8)? as usize;
    let manifest_size = read_header_u64(bytes, 16)? as usize;
    let manifest_end = manifest_offset
        .checked_add(manifest_size)
        .ok_or(ZrPackError::ManifestOutOfBounds)?;
    if manifest_offset < header_size() || manifest_end > bytes.len() {
        return Err(ZrPackError::ManifestOutOfBounds);
    }
    if manifest_end != bytes.len() {
        return Err(ZrPackError::ManifestTrailingBytes);
    }
    serde_json::from_slice(&bytes[manifest_offset..manifest_end])
        .map_err(|error| ZrPackError::ManifestDecode(error.to_string()))
}

pub(crate) fn validate_chunk_payload_extent(
    bytes: &[u8],
    chunks: &[ZrChunkEntry],
) -> Result<(), ZrPackError> {
    let manifest_offset = manifest_offset(bytes)?;
    let payload_end = chunk_payload_end(chunks)?;
    if manifest_offset != payload_end {
        return Err(ZrPackError::PayloadExtentMismatch);
    }
    Ok(())
}

fn manifest_offset(bytes: &[u8]) -> Result<usize, ZrPackError> {
    if bytes.len() < header_size() {
        return Err(ZrPackError::HeaderTooSmall);
    }
    usize::try_from(read_header_u64(bytes, 8)?).map_err(|_| ZrPackError::ManifestOutOfBounds)
}

fn chunk_payload_end(chunks: &[ZrChunkEntry]) -> Result<usize, ZrPackError> {
    let mut chunks = chunks.iter().collect::<Vec<_>>();
    chunks.sort_by(|left, right| left.offset.cmp(&right.offset));

    let mut payload_end = header_size();
    for chunk in chunks {
        let offset =
            usize::try_from(chunk.offset).map_err(|_| ZrPackError::PayloadExtentMismatch)?;
        let size = usize::try_from(chunk.size).map_err(|_| ZrPackError::PayloadExtentMismatch)?;
        if offset != payload_end {
            return Err(ZrPackError::PayloadExtentMismatch);
        }
        payload_end = payload_end
            .checked_add(size)
            .ok_or(ZrPackError::SizeOverflow)?;
    }
    Ok(payload_end)
}

fn validate_manifest_chunks(
    bytes: &[u8],
    manifest: &ZrPackDocumentManifest,
) -> Result<(), ZrPackError> {
    for chunk in &manifest.pack.chunks {
        let path = || {
            manifest
                .assets
                .iter()
                .find(|asset| asset.chunk_hash == chunk.hash)
                .map(|asset| asset.path.as_str())
                .unwrap_or("<chunk>")
                .to_string()
        };
        let chunk_bytes =
            chunk_range_bytes(bytes, chunk).ok_or_else(|| ZrPackError::ChunkOutOfBounds(path()))?;
        if zrpack_content_hash(chunk_bytes) != chunk.hash {
            return Err(ZrPackError::ChunkHashMismatch(path()));
        }
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
    let chunk_bytes = chunk_range_bytes(bytes, chunk)
        .ok_or_else(|| ZrPackError::ChunkOutOfBounds(path.into()))?;
    if zrpack_content_hash(chunk_bytes) != chunk.hash {
        return Err(ZrPackError::ChunkHashMismatch(path.to_string()));
    }
    Ok(chunk_bytes.to_vec())
}

fn chunk_range_bytes<'a>(bytes: &'a [u8], chunk: &ZrChunkEntry) -> Option<&'a [u8]> {
    let start = usize::try_from(chunk.offset).ok()?;
    let size = usize::try_from(chunk.size).ok()?;
    let end = start.checked_add(size)?;
    (start >= header_size() && end <= bytes.len()).then(|| &bytes[start..end])
}

pub(crate) fn read_header_u32(bytes: &[u8], offset: usize) -> Result<u32, ZrPackError> {
    Ok(u32::from_le_bytes(read_header_bytes::<4>(bytes, offset)?))
}

pub(crate) fn read_header_u64(bytes: &[u8], offset: usize) -> Result<u64, ZrPackError> {
    Ok(u64::from_le_bytes(read_header_bytes::<8>(bytes, offset)?))
}

fn read_header_bytes<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], ZrPackError> {
    let end = offset.checked_add(N).ok_or(ZrPackError::HeaderTooSmall)?;
    let range = bytes.get(offset..end).ok_or(ZrPackError::HeaderTooSmall)?;
    let mut value = [0; N];
    for (output, input) in value.iter_mut().zip(range.iter().copied()) {
        *output = input;
    }
    Ok(value)
}
