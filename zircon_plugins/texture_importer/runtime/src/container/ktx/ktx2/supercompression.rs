use std::io::Read;

use crate::container::support::{
    KTX2_HEADER_SIZE, KTX2_LEVEL_INDEX_ENTRY_SIZE, KTX2_SUPERCOMPRESSION_NONE,
    KTX2_SUPERCOMPRESSION_ZLIB, KTX2_SUPERCOMPRESSION_ZSTANDARD, parse_error, parse_error_value,
    read_u32_le, read_u64_le,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};

pub(super) fn expand_standard_supercompressed_levels(
    context: &AssetImportContext,
    level_count: u32,
    supercompression: u32,
) -> Result<Option<Vec<u8>>, AssetImportError> {
    if !matches!(
        supercompression,
        KTX2_SUPERCOMPRESSION_ZSTANDARD | KTX2_SUPERCOMPRESSION_ZLIB
    ) {
        return Ok(None);
    }

    let mut rewritten = ktx2_metadata_prefix(context, level_count)?;
    write_u32_le(&mut rewritten, 44, KTX2_SUPERCOMPRESSION_NONE, context)?;
    for level_index in 0..level_count {
        let entry_offset = level_entry_offset(context, level_index)?;
        let compressed_offset =
            usize::try_from(read_u64_le(context, entry_offset)?).map_err(|_| {
                parse_error_value(
                    context,
                    format!("ktx2 level {level_index} payload offset overflows usize"),
                )
            })?;
        let compressed_length =
            usize::try_from(read_u64_le(context, entry_offset + 8)?).map_err(|_| {
                parse_error_value(
                    context,
                    format!("ktx2 level {level_index} payload length overflows usize"),
                )
            })?;
        let expected_length =
            usize::try_from(read_u64_le(context, entry_offset + 16)?).map_err(|_| {
                parse_error_value(
                    context,
                    format!("ktx2 level {level_index} decoded length overflows usize"),
                )
            })?;
        if compressed_length == 0 {
            write_level_index_entry(&mut rewritten, entry_offset, 0, 0, context)?;
            continue;
        }
        let compressed_end = compressed_offset
            .checked_add(compressed_length)
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    format!("ktx2 level {level_index} compressed range overflows usize"),
                )
            })?;
        let compressed = context
            .source_bytes
            .get(compressed_offset..compressed_end)
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    format!("ktx2 level {level_index} compressed payload is truncated"),
                )
            })?;
        let decoded = decode_standard_supercompressed_level(
            context,
            level_index,
            supercompression,
            compressed,
            expected_length,
        )?;
        let aligned_len = align_eight(rewritten.len()).ok_or_else(|| {
            parse_error_value(
                context,
                format!("ktx2 level {level_index} decoded payload alignment overflows usize"),
            )
        })?;
        rewritten.resize(aligned_len, 0);
        let decoded_offset = rewritten.len();
        rewritten.extend_from_slice(&decoded);
        write_level_index_entry(
            &mut rewritten,
            entry_offset,
            decoded_offset,
            decoded.len(),
            context,
        )?;
    }
    Ok(Some(rewritten))
}

fn ktx2_metadata_prefix(
    context: &AssetImportContext,
    level_count: u32,
) -> Result<Vec<u8>, AssetImportError> {
    let level_index_end = level_entry_offset(context, level_count)?;
    let mut rewritten = context
        .source_bytes
        .get(..level_index_end)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            parse_error_value(
                context,
                "ktx2 level index is outside the container after validation",
            )
        })?;
    let data_format_descriptor = ktx2_metadata_section(context, 48, 52, "data format descriptor")?;
    let data_format_descriptor_offset = append_aligned_metadata(
        context,
        &mut rewritten,
        data_format_descriptor,
        "data format descriptor",
    )?;
    write_u32_le(
        &mut rewritten,
        48,
        u32::try_from(data_format_descriptor_offset).map_err(|_| {
            parse_error_value(
                context,
                "rewritten ktx2 data format descriptor offset overflows u32",
            )
        })?,
        context,
    )?;
    write_u32_le(
        &mut rewritten,
        52,
        u32::try_from(data_format_descriptor.len()).map_err(|_| {
            parse_error_value(
                context,
                "rewritten ktx2 data format descriptor length overflows u32",
            )
        })?,
        context,
    )?;
    let key_value_data = ktx2_metadata_section(context, 56, 60, "key/value data")?;
    if key_value_data.is_empty() {
        write_u32_le(&mut rewritten, 56, 0, context)?;
        write_u32_le(&mut rewritten, 60, 0, context)?;
    } else {
        let key_value_data_offset =
            append_aligned_metadata(context, &mut rewritten, key_value_data, "key/value data")?;
        write_u32_le(
            &mut rewritten,
            56,
            u32::try_from(key_value_data_offset).map_err(|_| {
                parse_error_value(
                    context,
                    "rewritten ktx2 key/value data offset overflows u32",
                )
            })?,
            context,
        )?;
        write_u32_le(
            &mut rewritten,
            60,
            u32::try_from(key_value_data.len()).map_err(|_| {
                parse_error_value(
                    context,
                    "rewritten ktx2 key/value data length overflows u32",
                )
            })?,
            context,
        )?;
    }
    write_u64_le(&mut rewritten, 64, 0, context)?;
    write_u64_le(&mut rewritten, 72, 0, context)?;
    Ok(rewritten)
}

fn ktx2_metadata_section<'a>(
    context: &'a AssetImportContext,
    offset_field: usize,
    length_field: usize,
    label: &str,
) -> Result<&'a [u8], AssetImportError> {
    let offset = usize::try_from(read_u32_le(context, offset_field)?)
        .map_err(|_| parse_error_value(context, format!("ktx2 {label} offset overflows usize")))?;
    let length = usize::try_from(read_u32_le(context, length_field)?)
        .map_err(|_| parse_error_value(context, format!("ktx2 {label} length overflows usize")))?;
    let end = offset
        .checked_add(length)
        .ok_or_else(|| parse_error_value(context, format!("ktx2 {label} range overflows usize")))?;
    context.source_bytes.get(offset..end).ok_or_else(|| {
        parse_error_value(
            context,
            format!("ktx2 {label} is outside the container after validation"),
        )
    })
}

fn append_aligned_metadata(
    context: &AssetImportContext,
    rewritten: &mut Vec<u8>,
    metadata: &[u8],
    label: &str,
) -> Result<usize, AssetImportError> {
    let aligned_len = align_four(rewritten.len()).ok_or_else(|| {
        parse_error_value(
            context,
            format!("rewritten ktx2 {label} alignment overflows usize"),
        )
    })?;
    rewritten.resize(aligned_len, 0);
    let offset = rewritten.len();
    rewritten.extend_from_slice(metadata);
    Ok(offset)
}

pub(super) fn level_entry_offset(
    context: &AssetImportContext,
    level_index: u32,
) -> Result<usize, AssetImportError> {
    let level_index = usize::try_from(level_index)
        .map_err(|_| parse_error_value(context, "ktx2 level index does not fit usize"))?;
    KTX2_HEADER_SIZE
        .checked_add(
            level_index
                .checked_mul(KTX2_LEVEL_INDEX_ENTRY_SIZE)
                .ok_or_else(|| {
                    parse_error_value(context, "ktx2 level index entry offset overflows usize")
                })?,
        )
        .ok_or_else(|| parse_error_value(context, "ktx2 level index entry offset overflows usize"))
}

fn decode_standard_supercompressed_level(
    context: &AssetImportContext,
    level_index: u32,
    supercompression: u32,
    compressed: &[u8],
    expected_length: usize,
) -> Result<Vec<u8>, AssetImportError> {
    match supercompression {
        KTX2_SUPERCOMPRESSION_ZSTANDARD => {
            let decoder = zstd::stream::read::Decoder::new(compressed).map_err(|error| {
                parse_error_value(
                    context,
                    format!("decode KTX2 zstd level {level_index}: {error}"),
                )
            })?;
            read_decoded_level(context, level_index, decoder, expected_length)
        }
        KTX2_SUPERCOMPRESSION_ZLIB => {
            let decoder = flate2::read::ZlibDecoder::new(compressed);
            read_decoded_level(context, level_index, decoder, expected_length)
        }
        _ => parse_error(
            context,
            format!(
                "ktx2 level {level_index} uses unsupported standard supercompression {supercompression}"
            ),
        ),
    }
}

fn read_decoded_level(
    context: &AssetImportContext,
    level_index: u32,
    mut reader: impl Read,
    expected_length: usize,
) -> Result<Vec<u8>, AssetImportError> {
    let limit = expected_length.checked_add(1).ok_or_else(|| {
        parse_error_value(
            context,
            format!("ktx2 level {level_index} decoded length limit overflows usize"),
        )
    })?;
    let mut decoded = Vec::new();
    reader
        .by_ref()
        .take(u64::try_from(limit).map_err(|_| {
            parse_error_value(
                context,
                format!("ktx2 level {level_index} decoded length limit overflows u64"),
            )
        })?)
        .read_to_end(&mut decoded)
        .map_err(|error| {
            parse_error_value(context, format!("decode KTX2 level {level_index}: {error}"))
        })?;
    if decoded.len() != expected_length {
        return parse_error(
            context,
            format!(
                "decode KTX2 level {level_index}: expected {expected_length} bytes, got {}",
                decoded.len()
            ),
        );
    }
    Ok(decoded)
}

fn align_eight(value: usize) -> Option<usize> {
    value.checked_add(7).map(|value| value & !7)
}

fn align_four(value: usize) -> Option<usize> {
    value.checked_add(3).map(|value| value & !3)
}

fn write_level_index_entry(
    bytes: &mut [u8],
    entry_offset: usize,
    offset: usize,
    length: usize,
    context: &AssetImportContext,
) -> Result<(), AssetImportError> {
    let offset = u64::try_from(offset)
        .map_err(|_| parse_error_value(context, "rewritten ktx2 level offset overflows u64"))?;
    let length = u64::try_from(length)
        .map_err(|_| parse_error_value(context, "rewritten ktx2 level length overflows u64"))?;
    write_u64_le(bytes, entry_offset, offset, context)?;
    write_u64_le(bytes, entry_offset + 8, length, context)?;
    write_u64_le(bytes, entry_offset + 16, length, context)
}

fn write_u32_le(
    bytes: &mut [u8],
    offset: usize,
    value: u32,
    context: &AssetImportContext,
) -> Result<(), AssetImportError> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| parse_error_value(context, "rewritten ktx2 u32 offset overflows usize"))?;
    let destination = bytes.get_mut(offset..end).ok_or_else(|| {
        parse_error_value(context, "rewritten ktx2 u32 range is outside the container")
    })?;
    destination.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u64_le(
    bytes: &mut [u8],
    offset: usize,
    value: u64,
    context: &AssetImportContext,
) -> Result<(), AssetImportError> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| parse_error_value(context, "rewritten ktx2 u64 offset overflows usize"))?;
    let destination = bytes.get_mut(offset..end).ok_or_else(|| {
        parse_error_value(context, "rewritten ktx2 u64 range is outside the container")
    })?;
    destination.copy_from_slice(&value.to_le_bytes());
    Ok(())
}
