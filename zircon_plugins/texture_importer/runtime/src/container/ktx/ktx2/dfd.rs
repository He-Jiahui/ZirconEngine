use crate::container::support::{parse_error, parse_error_value, read_u32_le};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};

// A KTX2 DFD starts with dfdTotalSize, then at least one 24-byte basic descriptor block.
const KTX2_DFD_MIN_BYTE_LENGTH: u32 = 16;
const KTX2_DFD_WORD_ALIGNMENT: u32 = 4;
const KTX2_DFD_TOTAL_SIZE_FIELD_BYTES: u32 = 4;
const KTX2_DFD_FIRST_DESCRIPTOR_BLOCK_OFFSET: usize = 4;
const KTX2_DFD_DESCRIPTOR_BLOCK_SIZE_OFFSET: usize = 4;
const KTX2_DFD_DESCRIPTOR_BLOCK_SIZE_FIELD_BYTES: u32 = 8;
// The descriptor-block size word stores the DFD version in its low 16 bits.
const KTX2_DFD_DESCRIPTOR_BLOCK_VERSION_MASK: u32 = 0xffff;
const KTX2_DFD_DESCRIPTOR_BLOCK_SIZE_SHIFT: u32 = 16;
const KTX2_DFD_COLOR_MODEL_WORD_OFFSET: usize = 8;
const KTX2_DFD_TRANSFER_SHIFT: u32 = 16;
const KTX2_DFD_TRANSFER_MASK: u32 = 0xff;
const KTX2_DFD_VERSION_NUMBER_1_3_OR_1_4: u32 = 2;
const KTX2_BASIC_DFD_DESCRIPTOR_BLOCK_MIN_SIZE: u32 = 24;
const KTX2_BASIC_DFD_SAMPLE_ALIGNMENT: u32 = 16;
const KTX2_DFD_MAX_TRANSFER_HLG_UNNORMALIZED_OETF: u32 = 19;

pub(super) fn validate_data_format_descriptor_header(
    context: &AssetImportContext,
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
) -> Result<(), AssetImportError> {
    if dfd_byte_offset == 0 || dfd_byte_length == 0 {
        return parse_error(context, "ktx2 data format descriptor must be present");
    }
    if dfd_byte_length < KTX2_DFD_MIN_BYTE_LENGTH {
        return parse_error(
            context,
            "ktx2 data format descriptor length must be at least 16 bytes",
        );
    }
    if dfd_byte_offset % KTX2_DFD_WORD_ALIGNMENT != 0 {
        return parse_error(
            context,
            format!(
                "ktx2 data format descriptor offset must be 4-byte aligned, got {dfd_byte_offset}"
            ),
        );
    }
    if dfd_byte_length % KTX2_DFD_WORD_ALIGNMENT != 0 {
        return parse_error(
            context,
            format!(
                "ktx2 data format descriptor length must be 4-byte aligned, got {dfd_byte_length}"
            ),
        );
    }
    Ok(())
}

pub(super) fn validate_data_format_descriptor(
    context: &AssetImportContext,
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
) -> Result<(), AssetImportError> {
    let dfd_offset = usize::try_from(dfd_byte_offset).map_err(|_| {
        parse_error_value(
            context,
            "ktx2 data format descriptor offset overflows usize",
        )
    })?;
    let dfd_total_size = read_u32_le(context, dfd_offset)?;
    if dfd_total_size != dfd_byte_length {
        return parse_error(
            context,
            format!(
                "ktx2 data format descriptor total size {dfd_total_size} must equal dfdByteLength {dfd_byte_length}"
            ),
        );
    }
    validate_data_format_descriptor_block_chain(context, dfd_offset, dfd_byte_length)?;
    Ok(())
}

fn validate_data_format_descriptor_block_chain(
    context: &AssetImportContext,
    dfd_offset: usize,
    dfd_byte_length: u32,
) -> Result<(), AssetImportError> {
    let mut remaining_descriptor_bytes = dfd_byte_length
        .checked_sub(KTX2_DFD_TOTAL_SIZE_FIELD_BYTES)
        .ok_or_else(|| {
            parse_error_value(
                context,
                "ktx2 data format descriptor length range underflows u32",
            )
        })?;
    let mut descriptor_block_offset = dfd_offset
        .checked_add(KTX2_DFD_FIRST_DESCRIPTOR_BLOCK_OFFSET)
        .ok_or_else(|| {
            parse_error_value(
                context,
                "ktx2 data format descriptor block offset overflows usize",
            )
        })?;
    let mut descriptor_block_index = 0_u32;

    while remaining_descriptor_bytes > 0 {
        if remaining_descriptor_bytes < KTX2_DFD_DESCRIPTOR_BLOCK_SIZE_FIELD_BYTES {
            return parse_error(
                context,
                format!(
                    "ktx2 data format descriptor block chain leaves {remaining_descriptor_bytes} trailing descriptor bytes"
                ),
            );
        }

        let vendor_and_type = read_u32_le(context, descriptor_block_offset)?;
        if vendor_and_type != 0 {
            return parse_error(
                context,
                format!(
                    "ktx2 data format descriptor block {descriptor_block_index} vendor/type word must be 0"
                ),
            );
        }
        let descriptor_block_size_word = read_u32_le(
            context,
            descriptor_block_offset
                .checked_add(KTX2_DFD_DESCRIPTOR_BLOCK_SIZE_OFFSET)
                .ok_or_else(|| {
                    parse_error_value(
                        context,
                        "ktx2 data format descriptor block size offset overflows usize",
                    )
                })?,
        )?;
        let descriptor_block_version =
            descriptor_block_size_word & KTX2_DFD_DESCRIPTOR_BLOCK_VERSION_MASK;
        let descriptor_block_size =
            descriptor_block_size_word >> KTX2_DFD_DESCRIPTOR_BLOCK_SIZE_SHIFT;
        validate_basic_data_format_descriptor_block_size(context, descriptor_block_size)?;
        validate_data_format_descriptor_block_size_fits(
            context,
            descriptor_block_index,
            descriptor_block_size,
            remaining_descriptor_bytes,
            dfd_byte_length,
        )?;
        validate_data_format_descriptor_block_version(
            context,
            descriptor_block_index,
            descriptor_block_version,
        )?;
        validate_data_format_descriptor_block_transfer(context, descriptor_block_offset)?;
        let descriptor_block_size = usize::try_from(descriptor_block_size).map_err(|_| {
            parse_error_value(
                context,
                "ktx2 data format descriptor block size overflows usize",
            )
        })?;
        descriptor_block_offset = descriptor_block_offset
            .checked_add(descriptor_block_size)
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    "ktx2 data format descriptor block offset overflows usize",
                )
            })?;
        remaining_descriptor_bytes -= u32::try_from(descriptor_block_size).map_err(|_| {
            parse_error_value(
                context,
                "ktx2 data format descriptor block size overflows u32",
            )
        })?;
        descriptor_block_index += 1;
    }
    Ok(())
}

fn validate_data_format_descriptor_block_version(
    context: &AssetImportContext,
    descriptor_block_index: u32,
    descriptor_block_version: u32,
) -> Result<(), AssetImportError> {
    if descriptor_block_version == KTX2_DFD_VERSION_NUMBER_1_3_OR_1_4 {
        return Ok(());
    }
    parse_error(
        context,
        format!(
            "ktx2 data format descriptor block {descriptor_block_index} version must be 2, got {descriptor_block_version}"
        ),
    )
}

fn validate_data_format_descriptor_block_transfer(
    context: &AssetImportContext,
    descriptor_block_offset: usize,
) -> Result<(), AssetImportError> {
    let color_model_word = read_u32_le(
        context,
        descriptor_block_offset
            .checked_add(KTX2_DFD_COLOR_MODEL_WORD_OFFSET)
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    "ktx2 data format descriptor color model word offset overflows usize",
                )
            })?,
    )?;
    let transfer = (color_model_word >> KTX2_DFD_TRANSFER_SHIFT) & KTX2_DFD_TRANSFER_MASK;
    if transfer <= KTX2_DFD_MAX_TRANSFER_HLG_UNNORMALIZED_OETF {
        return Ok(());
    }
    parse_error(
        context,
        format!("ktx2 data format descriptor transfer function {transfer} is not supported"),
    )
}

fn validate_data_format_descriptor_block_size_fits(
    context: &AssetImportContext,
    descriptor_block_index: u32,
    descriptor_block_size: u32,
    remaining_descriptor_bytes: u32,
    dfd_byte_length: u32,
) -> Result<(), AssetImportError> {
    if descriptor_block_size <= remaining_descriptor_bytes {
        return Ok(());
    }
    if descriptor_block_index == 0 {
        return parse_error(
            context,
            format!(
                "ktx2 data format descriptor basic descriptor block size {descriptor_block_size} exceeds dfdByteLength {dfd_byte_length}"
            ),
        );
    }
    parse_error(
        context,
        format!(
            "ktx2 data format descriptor block {descriptor_block_index} size {descriptor_block_size} exceeds remaining DFD descriptor bytes {remaining_descriptor_bytes}"
        ),
    )
}

fn validate_basic_data_format_descriptor_block_size(
    context: &AssetImportContext,
    descriptor_block_size: u32,
) -> Result<(), AssetImportError> {
    if descriptor_block_size < KTX2_BASIC_DFD_DESCRIPTOR_BLOCK_MIN_SIZE
        || (descriptor_block_size - KTX2_BASIC_DFD_DESCRIPTOR_BLOCK_MIN_SIZE)
            % KTX2_BASIC_DFD_SAMPLE_ALIGNMENT
            != 0
    {
        return parse_error(
            context,
            format!(
                "ktx2 data format descriptor basic descriptor block size {descriptor_block_size} must be at least 24 bytes and 16-byte sample aligned"
            ),
        );
    }
    Ok(())
}
