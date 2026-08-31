use crate::core::math::UVec2;
use crate::text::sdf::{SdfBakeParams, SdfMode};

use super::{
    SdfOfflineArtifact, SdfOfflineArtifactError, SdfOfflineArtifactIdentity, SdfOfflineGlyph,
    SdfOfflineGlyphMetrics, SdfOfflinePage, SdfOfflineRect,
};

const MAGIC: [u8; 8] = *b"ZRZSDF\0\0";
const FORMAT_VERSION: u32 = 1;
const HEADER_LEN: usize = 208;
const CHECKSUM_OFFSET: usize = 176;
const CHECKSUM_LEN: usize = 32;
const PAGE_RECORD_LEN: usize = 20;
const GLYPH_RECORD_LEN: usize = 48;

pub(super) fn encode(artifact: &SdfOfflineArtifact) -> Result<Vec<u8>, SdfOfflineArtifactError> {
    let page_count = u32::try_from(artifact.pages().len())
        .map_err(|_| SdfOfflineArtifactError::LengthOverflow)?;
    let glyph_count = u32::try_from(artifact.glyphs().len())
        .map_err(|_| SdfOfflineArtifactError::LengthOverflow)?;
    let page_records_len = artifact
        .pages()
        .len()
        .checked_mul(PAGE_RECORD_LEN)
        .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
    let glyph_records_len = artifact
        .glyphs()
        .len()
        .checked_mul(GLYPH_RECORD_LEN)
        .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
    let pixel_bytes_len = artifact.pages().iter().try_fold(0_usize, |total, page| {
        total
            .checked_add(page.pixels.len())
            .ok_or(SdfOfflineArtifactError::LengthOverflow)
    })?;
    let total_len = HEADER_LEN
        .checked_add(page_records_len)
        .and_then(|len| len.checked_add(glyph_records_len))
        .and_then(|len| len.checked_add(pixel_bytes_len))
        .ok_or(SdfOfflineArtifactError::LengthOverflow)?;

    let identity = artifact.identity();
    let guid = identity.asset_guid.as_bytes();
    if guid.len() != 36 {
        return Err(SdfOfflineArtifactError::InvalidAssetGuid(
            identity.asset_guid.clone(),
        ));
    }

    let mut bytes = Vec::with_capacity(total_len);
    bytes.extend_from_slice(&MAGIC);
    push_u32(&mut bytes, FORMAT_VERSION);
    push_u32(&mut bytes, HEADER_LEN as u32);
    bytes.extend_from_slice(guid);
    push_u32(&mut bytes, identity.face_index);
    bytes.extend_from_slice(identity.variation_hash.as_bytes());
    bytes.extend_from_slice(identity.source_hash.as_bytes());
    push_u32(&mut bytes, identity.params.mode.shader_discriminant());
    push_u32(&mut bytes, identity.params.bake_em_px);
    push_u32(&mut bytes, identity.params.spread_px_milli);
    push_u32(&mut bytes, artifact.page_size().x);
    push_u32(&mut bytes, artifact.page_size().y);
    push_u32(&mut bytes, page_count);
    push_u32(&mut bytes, glyph_count);
    push_u64(&mut bytes, as_u64(page_records_len)?);
    push_u64(&mut bytes, as_u64(glyph_records_len)?);
    push_u64(&mut bytes, as_u64(pixel_bytes_len)?);
    push_u32(&mut bytes, 0);
    debug_assert_eq!(bytes.len(), CHECKSUM_OFFSET);
    bytes.resize(HEADER_LEN, 0);

    let mut page_offset = 0_usize;
    for page in artifact.pages() {
        push_u32(&mut bytes, page.page_index);
        push_u64(&mut bytes, as_u64(page_offset)?);
        push_u64(&mut bytes, as_u64(page.pixels.len())?);
        page_offset = page_offset
            .checked_add(page.pixels.len())
            .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
    }
    for glyph in artifact.glyphs() {
        push_u32(&mut bytes, glyph.glyph_id);
        push_u32(&mut bytes, glyph.codepoint);
        push_u32(&mut bytes, glyph.page_index);
        push_u32(&mut bytes, glyph.rect.x);
        push_u32(&mut bytes, glyph.rect.y);
        push_u32(&mut bytes, glyph.rect.width);
        push_u32(&mut bytes, glyph.rect.height);
        push_f32(&mut bytes, glyph.metrics.bitmap_left);
        push_f32(&mut bytes, glyph.metrics.bitmap_bottom);
        push_f32(&mut bytes, glyph.metrics.advance);
        push_f32(&mut bytes, glyph.metrics.ascent);
        push_u32(&mut bytes, 0);
    }
    for page in artifact.pages() {
        bytes.extend_from_slice(&page.pixels);
    }
    debug_assert_eq!(bytes.len(), total_len);

    let checksum = checksum(&bytes);
    bytes[CHECKSUM_OFFSET..CHECKSUM_OFFSET + CHECKSUM_LEN].copy_from_slice(&checksum);
    Ok(bytes)
}

pub(super) fn decode(bytes: &[u8]) -> Result<SdfOfflineArtifact, SdfOfflineArtifactError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_array::<8>()? != MAGIC {
        return Err(SdfOfflineArtifactError::InvalidMagic);
    }
    let version = cursor.read_u32()?;
    if version != FORMAT_VERSION {
        return Err(SdfOfflineArtifactError::UnsupportedVersion {
            found: version,
            supported: FORMAT_VERSION,
        });
    }
    let header_len = cursor.read_u32()?;
    if header_len as usize != HEADER_LEN {
        return Err(SdfOfflineArtifactError::InvalidHeaderLength(header_len));
    }
    let guid_bytes = cursor.read_array::<36>()?;
    let asset_guid = std::str::from_utf8(&guid_bytes)
        .map_err(|_| SdfOfflineArtifactError::InvalidAssetGuid("<non-utf8>".to_string()))?
        .to_string();
    let face_index = cursor.read_u32()?;
    let variation_hash = cursor.read_array::<32>()?;
    let source_hash = cursor.read_array::<32>()?;
    let mode_value = cursor.read_u32()?;
    let mode = SdfMode::from_shader_discriminant(mode_value)
        .ok_or(SdfOfflineArtifactError::InvalidMode(mode_value))?;
    let bake_em_px = cursor.read_u32()?;
    let spread_px_milli = cursor.read_u32()?;
    let page_size = UVec2::new(cursor.read_u32()?, cursor.read_u32()?);
    let page_count = cursor.read_u32()?;
    let glyph_count = cursor.read_u32()?;
    let page_records_len = as_usize(cursor.read_u64()?)?;
    let glyph_records_len = as_usize(cursor.read_u64()?)?;
    let pixel_bytes_len = as_usize(cursor.read_u64()?)?;
    if cursor.read_u32()? != 0 {
        return Err(SdfOfflineArtifactError::NonZeroReserved);
    }
    let stored_checksum = cursor.read_array::<CHECKSUM_LEN>()?;
    debug_assert_eq!(cursor.offset, HEADER_LEN);

    let expected_page_records_len = usize::try_from(page_count)
        .ok()
        .and_then(|count| count.checked_mul(PAGE_RECORD_LEN))
        .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
    let expected_glyph_records_len = usize::try_from(glyph_count)
        .ok()
        .and_then(|count| count.checked_mul(GLYPH_RECORD_LEN))
        .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
    if page_records_len != expected_page_records_len
        || glyph_records_len != expected_glyph_records_len
    {
        return Err(SdfOfflineArtifactError::InvalidSectionLength);
    }
    let expected_total = HEADER_LEN
        .checked_add(page_records_len)
        .and_then(|len| len.checked_add(glyph_records_len))
        .and_then(|len| len.checked_add(pixel_bytes_len))
        .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
    if bytes.len() < expected_total {
        return Err(SdfOfflineArtifactError::UnexpectedEof);
    }
    if bytes.len() > expected_total {
        return Err(SdfOfflineArtifactError::TrailingBytes);
    }
    if checksum(bytes) != stored_checksum {
        return Err(SdfOfflineArtifactError::ChecksumMismatch);
    }

    let mut page_records = Vec::with_capacity(page_count as usize);
    let mut expected_offset = 0_usize;
    for _ in 0..page_count {
        let page_index = cursor.read_u32()?;
        let source_offset = as_usize(cursor.read_u64()?)?;
        let byte_len = as_usize(cursor.read_u64()?)?;
        if source_offset != expected_offset {
            return Err(SdfOfflineArtifactError::NonContiguousPagePayload);
        }
        expected_offset = expected_offset
            .checked_add(byte_len)
            .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
        page_records.push((page_index, source_offset, byte_len));
    }
    if expected_offset != pixel_bytes_len {
        return Err(SdfOfflineArtifactError::NonContiguousPagePayload);
    }

    let mut glyphs = Vec::with_capacity(glyph_count as usize);
    for _ in 0..glyph_count {
        let glyph_id = cursor.read_u32()?;
        let codepoint = cursor.read_u32()?;
        let page_index = cursor.read_u32()?;
        let rect = SdfOfflineRect::new(
            cursor.read_u32()?,
            cursor.read_u32()?,
            cursor.read_u32()?,
            cursor.read_u32()?,
        );
        let metrics = SdfOfflineGlyphMetrics {
            bitmap_left: cursor.read_f32()?,
            bitmap_bottom: cursor.read_f32()?,
            advance: cursor.read_f32()?,
            ascent: cursor.read_f32()?,
        };
        if cursor.read_u32()? != 0 {
            return Err(SdfOfflineArtifactError::NonZeroReserved);
        }
        glyphs.push(SdfOfflineGlyph {
            glyph_id,
            codepoint,
            page_index,
            rect,
            metrics,
        });
    }

    let pixel_start = cursor.offset;
    let mut pages = Vec::with_capacity(page_count as usize);
    for (page_index, source_offset, byte_len) in page_records {
        let start = pixel_start
            .checked_add(source_offset)
            .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
        let end = start
            .checked_add(byte_len)
            .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
        pages.push(SdfOfflinePage {
            page_index,
            pixels: bytes
                .get(start..end)
                .ok_or(SdfOfflineArtifactError::UnexpectedEof)?
                .to_vec(),
        });
    }

    SdfOfflineArtifact::new(
        SdfOfflineArtifactIdentity {
            asset_guid,
            face_index,
            variation_hash: variation_hash.into(),
            source_hash: source_hash.into(),
            params: SdfBakeParams {
                mode,
                bake_em_px,
                spread_px_milli,
            },
        },
        page_size,
        pages,
        glyphs,
    )
}

fn checksum(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    let prefix_end = CHECKSUM_OFFSET.min(bytes.len());
    hasher.update(&bytes[..prefix_end]);
    if bytes.len() > HEADER_LEN {
        hasher.update(&bytes[HEADER_LEN..]);
    }
    *hasher.finalize().as_bytes()
}

fn as_u64(value: usize) -> Result<u64, SdfOfflineArtifactError> {
    u64::try_from(value).map_err(|_| SdfOfflineArtifactError::LengthOverflow)
}

fn as_usize(value: u64) -> Result<usize, SdfOfflineArtifactError> {
    usize::try_from(value).map_err(|_| SdfOfflineArtifactError::LengthOverflow)
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(bytes: &mut Vec<u8>, value: f32) {
    push_u32(bytes, value.to_bits());
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], SdfOfflineArtifactError> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or(SdfOfflineArtifactError::LengthOverflow)?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(SdfOfflineArtifactError::UnexpectedEof)?;
        self.offset = end;
        slice
            .try_into()
            .map_err(|_| SdfOfflineArtifactError::UnexpectedEof)
    }

    fn read_u32(&mut self) -> Result<u32, SdfOfflineArtifactError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    fn read_u64(&mut self) -> Result<u64, SdfOfflineArtifactError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    fn read_f32(&mut self) -> Result<f32, SdfOfflineArtifactError> {
        Ok(f32::from_bits(self.read_u32()?))
    }
}
