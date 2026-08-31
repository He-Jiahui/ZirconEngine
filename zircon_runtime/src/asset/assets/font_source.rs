use std::error::Error;
use std::io::Cursor;

use thiserror::Error;

use super::FontAssetSourceFormat;
pub use budget::FontSourceBudgetError;
use budget::{
    validate_decoded_bytes, validate_font_metadata_with_budget, validate_source_bytes,
    validate_source_file_len, validate_standalone_face_bytes_with_budget,
    validate_woff2_declared_decoded_bytes, FontSourceBudget, FONT_SOURCE_BUDGET,
};

mod budget;

const SFNT_CHECKSUM_MAGIC: u32 = 0xB1B0_AFBA;

/// Decoded font bytes plus the source container format recorded in asset metadata.
#[derive(Clone, Debug)]
pub(crate) struct DecodedFontSource {
    bytes: Vec<u8>,
    source_format: FontAssetSourceFormat,
}

impl DecodedFontSource {
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub(crate) fn source_format(&self) -> FontAssetSourceFormat {
        self.source_format
    }
}

/// Opaque WOFF2 decode failure that preserves the third-party source error and
/// contains malformed-input panics at the decoder isolation boundary.
#[derive(Debug, Error)]
pub enum FontSourceDecodeError {
    #[error(transparent)]
    Budget(#[from] FontSourceBudgetError),
    #[error("WOFF2 font source decode failed: {source}")]
    Decoder {
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("WOFF2 font source decode failed because the decoder panicked")]
    DecoderPanic,
}

impl FontSourceDecodeError {
    fn new(source: impl Error + Send + Sync + 'static) -> Self {
        Self::Decoder {
            source: Box::new(source),
        }
    }
}

/// Opaque per-face metadata parse failure with the failing collection index.
#[derive(Debug, Error)]
#[error("font face {face_index} metadata parse failed: {source}")]
pub struct FontMetadataParseError {
    face_index: u32,
    #[source]
    source: Box<dyn Error + Send + Sync>,
}

#[derive(Debug, Error)]
pub(crate) enum FontFaceExtractionError {
    #[error(transparent)]
    Budget(#[from] FontSourceBudgetError),
    #[error("font face {face_index} is invalid: {source}")]
    InvalidFace {
        face_index: u32,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("font collection face {face_index} is malformed: {reason}")]
    Malformed {
        face_index: u32,
        reason: &'static str,
    },
}

impl FontMetadataParseError {
    pub(crate) fn new(face_index: u32, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            face_index,
            source: Box::new(source),
        }
    }

    pub(crate) fn budget(source: FontSourceBudgetError) -> Self {
        Self {
            face_index: source.face_index().unwrap_or(0),
            source: Box::new(source),
        }
    }

    pub fn face_index(&self) -> u32 {
        self.face_index
    }
}

/// Decode a webfont container once at the asset boundary. All downstream font
/// consumers receive SFNT/TTC bytes and never depend on WOFF2 implementation types.
pub(crate) fn decode_font_source(
    bytes: Vec<u8>,
) -> Result<DecodedFontSource, FontSourceDecodeError> {
    decode_font_source_with_budget(bytes, FONT_SOURCE_BUDGET)
}

fn decode_font_source_with_budget(
    bytes: Vec<u8>,
    budget: FontSourceBudget,
) -> Result<DecodedFontSource, FontSourceDecodeError> {
    validate_source_bytes(&bytes, budget)?;
    if !woff2_patched::decode::is_woff2(&bytes) {
        validate_decoded_bytes(&bytes, budget)?;
        return Ok(DecodedFontSource {
            bytes,
            source_format: FontAssetSourceFormat::Sfnt,
        });
    }
    validate_woff2_declared_decoded_bytes(&bytes, budget)?;

    let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        woff2_patched::convert_woff2_to_ttf(&mut Cursor::new(bytes))
    }))
    .map_err(|_| FontSourceDecodeError::DecoderPanic)?
    .map_err(FontSourceDecodeError::new)?;
    validate_decoded_bytes(&decoded, budget)?;
    Ok(DecodedFontSource {
        bytes: decoded,
        source_format: FontAssetSourceFormat::Woff2,
    })
}

pub(crate) fn validate_font_metadata_budget(bytes: &[u8]) -> Result<(), FontSourceBudgetError> {
    validate_font_metadata_with_budget(bytes, FONT_SOURCE_BUDGET)
}

pub(crate) fn validate_font_source_file_len(file_len: u64) -> Result<(), FontSourceBudgetError> {
    validate_source_file_len(file_len)
}

pub(crate) fn font_cmap_range_budget() -> usize {
    budget::max_cmap_ranges()
}

pub(crate) fn validate_standalone_face_bytes(
    bytes: &[u8],
    face_index: u32,
) -> Result<(), FontSourceBudgetError> {
    validate_standalone_face_bytes_with_budget(bytes, face_index, FONT_SOURCE_BUDGET)
}

/// Materialize one TTC face as a standalone SFNT buffer for backends that do
/// not expose a collection-index setting. Table data remains unchanged except
/// for the required standalone `head.checkSumAdjustment` recomputation.
pub(crate) fn standalone_sfnt_face(
    bytes: &[u8],
    face_index: u32,
) -> Result<Vec<u8>, FontFaceExtractionError> {
    validate_font_metadata_budget(bytes).map_err(FontFaceExtractionError::Budget)?;
    validate_standalone_face_bytes(bytes, face_index).map_err(FontFaceExtractionError::Budget)?;
    ttf_parser::Face::parse(bytes, face_index).map_err(|source| {
        FontFaceExtractionError::InvalidFace {
            face_index,
            source: Box::new(source),
        }
    })?;
    if !bytes.starts_with(b"ttcf") {
        return Ok(bytes.to_vec());
    }

    let face_offset = read_u32(bytes, 12 + face_index as usize * 4)
        .map(|value| value as usize)
        .ok_or(FontFaceExtractionError::Malformed {
            face_index,
            reason: "face directory offset is missing",
        })?;
    let scaler_type =
        bytes
            .get(face_offset..face_offset + 4)
            .ok_or(FontFaceExtractionError::Malformed {
                face_index,
                reason: "face offset table is truncated",
            })?;
    let table_count = read_u16(bytes, face_offset + 4).map(usize::from).ok_or(
        FontFaceExtractionError::Malformed {
            face_index,
            reason: "table count is missing",
        },
    )?;
    let directory_end = face_offset.checked_add(12 + table_count * 16).ok_or(
        FontFaceExtractionError::Malformed {
            face_index,
            reason: "table directory length overflowed",
        },
    )?;
    if directory_end > bytes.len() {
        return Err(FontFaceExtractionError::Malformed {
            face_index,
            reason: "table directory is truncated",
        });
    }

    let mut output = vec![0; 12 + table_count * 16];
    output[0..4].copy_from_slice(scaler_type);
    output[4..12].copy_from_slice(&bytes[face_offset + 4..face_offset + 12]);
    let mut head_offset = None;

    for table_index in 0..table_count {
        let source_record = face_offset + 12 + table_index * 16;
        let target_record = 12 + table_index * 16;
        let tag: [u8; 4] = bytes
            .get(source_record..source_record + 4)
            .and_then(|value| value.try_into().ok())
            .ok_or(FontFaceExtractionError::Malformed {
                face_index,
                reason: "table tag is missing",
            })?;
        let source_offset = read_u32(bytes, source_record + 8)
            .map(|value| value as usize)
            .ok_or(FontFaceExtractionError::Malformed {
                face_index,
                reason: "table offset is missing",
            })?;
        let source_len = read_u32(bytes, source_record + 12)
            .map(|value| value as usize)
            .ok_or(FontFaceExtractionError::Malformed {
                face_index,
                reason: "table length is missing",
            })?;
        let source_end =
            source_offset
                .checked_add(source_len)
                .ok_or(FontFaceExtractionError::Malformed {
                    face_index,
                    reason: "table range overflowed",
                })?;
        let source_data =
            bytes
                .get(source_offset..source_end)
                .ok_or(FontFaceExtractionError::Malformed {
                    face_index,
                    reason: "table data is truncated",
                })?;

        pad_to_four(&mut output);
        let target_offset = output.len();
        output.extend_from_slice(source_data);
        if tag == *b"head" && source_data.len() >= 12 {
            output[target_offset + 8..target_offset + 12].fill(0);
            head_offset = Some(target_offset);
        }
        let target_end = target_offset + source_len;
        let table_checksum = sfnt_checksum(&output[target_offset..target_end]);
        output[target_record..target_record + 4].copy_from_slice(&tag);
        output[target_record + 4..target_record + 8].copy_from_slice(&table_checksum.to_be_bytes());
        output[target_record + 8..target_record + 12]
            .copy_from_slice(&(target_offset as u32).to_be_bytes());
        output[target_record + 12..target_record + 16]
            .copy_from_slice(&(source_len as u32).to_be_bytes());
    }

    if let Some(head_offset) = head_offset {
        let adjustment = SFNT_CHECKSUM_MAGIC.wrapping_sub(sfnt_checksum(&output));
        output[head_offset + 8..head_offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    }
    Ok(output)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn pad_to_four(bytes: &mut Vec<u8>) {
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
}

fn sfnt_checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |checksum, chunk| {
        let mut word = [0u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        checksum.wrapping_add(u32::from_be_bytes(word))
    })
}

#[cfg(test)]
mod tests;
