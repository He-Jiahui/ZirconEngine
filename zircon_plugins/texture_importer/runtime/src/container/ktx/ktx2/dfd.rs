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
            format_args!(
                "ktx2 data format descriptor offset must be 4-byte aligned, got {dfd_byte_offset}"
            ),
        );
    }
    if dfd_byte_length % KTX2_DFD_WORD_ALIGNMENT != 0 {
        return parse_error(
            context,
            format_args!(
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
            format_args!(
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
                format_args!(
                    "ktx2 data format descriptor block chain leaves {remaining_descriptor_bytes} trailing descriptor bytes"
                ),
            );
        }

        let vendor_and_type = read_u32_le(context, descriptor_block_offset)?;
        if vendor_and_type != 0 {
            return parse_error(
                context,
                format_args!(
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
        format_args!(
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
        format_args!("ktx2 data format descriptor transfer function {transfer} is not supported"),
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
            format_args!(
                "ktx2 data format descriptor basic descriptor block size {descriptor_block_size} exceeds dfdByteLength {dfd_byte_length}"
            ),
        );
    }
    parse_error(
        context,
        format_args!(
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
            format_args!(
                "ktx2 data format descriptor basic descriptor block size {descriptor_block_size} must be at least 24 bytes and 16-byte sample aligned"
            ),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::asset::AssetUri;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const ERRORS_PER_SAMPLE: usize = 8_192;

    #[test]
    fn borrowed_dfd_error_arguments_preserve_diagnostic_text() {
        let context = test_context();

        let error = validate_data_format_descriptor_header(&context, 2, 16).unwrap_err();

        assert_eq!(
            error.to_string(),
            "parse texture container broken.ktx2: ktx2 data format descriptor offset must be 4-byte aligned, got 2"
        );
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_borrowed_dfd_error_arguments() {
        let context = test_context();
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_errors(&context, legacy_error));
                optimized_raw.push(measure_errors(&context, optimized_error));
            } else {
                optimized_raw.push(measure_errors(&context, optimized_error));
                legacy_raw.push(measure_errors(&context, legacy_error));
            }
        }

        let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
        let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "borrowed DFD error arguments must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "PERF_RESULT task=plugins07_borrowed_dfd_error_arguments sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank errors_per_sample={ERRORS_PER_SAMPLE} legacy_allocations_per_error=2 optimized_allocations_per_error=1 legacy_detail_allocations_per_sample={ERRORS_PER_SAMPLE} optimized_detail_allocations_per_sample=0 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn legacy_error(context: &AssetImportContext, index: usize) -> AssetImportError {
        let detail = format!(
            "ktx2 data format descriptor block {} version must be 2, got {}",
            index % 8,
            3 + index % 17
        );
        parse_error_value(context, detail)
    }

    fn optimized_error(context: &AssetImportContext, index: usize) -> AssetImportError {
        parse_error_value(
            context,
            format_args!(
                "ktx2 data format descriptor block {} version must be 2, got {}",
                index % 8,
                3 + index % 17
            ),
        )
    }

    fn measure_errors(
        context: &AssetImportContext,
        make_error: fn(&AssetImportContext, usize) -> AssetImportError,
    ) -> u64 {
        let started = Instant::now();
        for index in 0..ERRORS_PER_SAMPLE {
            black_box(make_error(black_box(context), black_box(index)));
        }
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn test_context() -> AssetImportContext {
        AssetImportContext::new(
            "broken.ktx2".into(),
            AssetUri::parse("res://textures/broken.ktx2").unwrap(),
            vec![0; 16],
            Default::default(),
        )
    }

    fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u64]) -> String {
        let values = samples
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!("[{values}]")
    }
}
