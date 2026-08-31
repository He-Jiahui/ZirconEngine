use super::{
    checked_u64_range_end, ktx_four_byte_padding, level_index_end, read_face_count,
    texture_array_layers, texture_dimension_from_header, validate_3d_array_layers,
    validate_3d_height, validate_cubemap_2d_square_faces, validate_cubemap_depth,
    validate_mip_count_fits_extent, TextureContainerInfo,
};
use crate::container::support::{
    checked_layer_count, parse_error, parse_error_value, read_nonzero_u32, read_u32_le,
    read_u64_le, require_len, texture_depth_or_array_layers, KTX2_HEADER_SIZE, KTX2_IDENTIFIER,
    KTX2_LEVEL_INDEX_ENTRY_SIZE, KTX2_SUPERCOMPRESSION_BASIS_LZ, KTX2_SUPERCOMPRESSION_NONE,
    KTX2_SUPERCOMPRESSION_ZLIB, KTX2_SUPERCOMPRESSION_ZSTANDARD,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};

mod dfd;
mod supercompression;

use supercompression::{expand_standard_supercompressed_levels, level_entry_offset};

pub(super) fn parse(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, KTX2_HEADER_SIZE, "ktx2 header")?;
    if &bytes[..12] != KTX2_IDENTIFIER {
        return parse_error(context, "ktx2 header missing KTX 2 identifier");
    }

    let vk_format = read_u32_le(context, 12)?;
    let _type_size = read_nonzero_u32(context, 16, "ktx2 type size")?;
    let width = read_nonzero_u32(context, 20, "ktx2 width")?;
    let raw_height = read_u32_le(context, 24)?;
    let raw_depth = read_u32_le(context, 28)?;
    validate_3d_height(context, "ktx2", raw_height, raw_depth)?;
    let height = raw_height.max(1);
    let raw_layer_count = read_u32_le(context, 32)?;
    validate_3d_array_layers(context, "ktx2", raw_layer_count, raw_depth)?;
    let layer_count = raw_layer_count.max(1);
    let face_count = read_face_count(context, 36, "ktx2 face count")?;
    validate_cubemap_depth(context, "ktx2", face_count, raw_depth)?;
    validate_cubemap_2d_square_faces(context, "ktx2", face_count, width, raw_height)?;
    let level_count = read_u32_le(context, 40)?.max(1);
    validate_mip_count_fits_extent(
        context,
        "ktx2",
        width,
        height,
        raw_depth.max(1),
        level_count,
    )?;
    let supercompression = read_u32_le(context, 44)?;
    validate_supercompression_scheme(context, supercompression)?;
    validate_vk_format_supercompression_pair(context, vk_format, supercompression)?;
    let parsed_layers =
        checked_layer_count(context, "ktx2 array layer count", layer_count, face_count)?;
    let required_level_index_len = usize::try_from(level_count)
        .ok()
        .and_then(|count| count.checked_mul(KTX2_LEVEL_INDEX_ENTRY_SIZE))
        .and_then(|level_index_len| KTX2_HEADER_SIZE.checked_add(level_index_len))
        .ok_or_else(|| parse_error_value(context, "ktx2 level index length overflows usize"))?;
    require_len(context, required_level_index_len, "ktx2 level index")?;
    let level_index_end = level_index_end(context, level_count)?;
    let level_payloads = validate_level_payload_ranges(
        context,
        level_count,
        level_index_end,
        supercompression,
        parsed_layers,
    )?;
    let dfd_byte_offset = read_u32_le(context, 48)?;
    let dfd_byte_length = read_u32_le(context, 52)?;
    dfd::validate_data_format_descriptor_header(context, dfd_byte_offset, dfd_byte_length)?;
    require_metadata_range(
        context,
        u64::from(dfd_byte_offset),
        u64::from(dfd_byte_length),
        level_index_end,
        "ktx2 data format descriptor",
    )?;
    dfd::validate_data_format_descriptor(context, dfd_byte_offset, dfd_byte_length)?;
    let key_value_data_offset = read_u32_le(context, 56)?;
    let key_value_data_length = read_u32_le(context, 60)?;
    validate_zero_length_metadata_offset(
        context,
        u64::from(key_value_data_offset),
        u64::from(key_value_data_length),
        "ktx2 key/value data",
    )?;
    require_metadata_range(
        context,
        u64::from(key_value_data_offset),
        u64::from(key_value_data_length),
        level_index_end,
        "ktx2 key/value data",
    )?;
    let supercompression_global_data_offset = read_u64_le(context, 64)?;
    let supercompression_global_data_length = read_u64_le(context, 72)?;
    validate_zero_length_metadata_offset(
        context,
        supercompression_global_data_offset,
        supercompression_global_data_length,
        "ktx2 supercompression global data",
    )?;
    require_metadata_range(
        context,
        supercompression_global_data_offset,
        supercompression_global_data_length,
        level_index_end,
        "ktx2 supercompression global data",
    )?;
    validate_supercompression_global_data(
        context,
        supercompression,
        supercompression_global_data_length,
        level_payloads.has_non_empty_payload,
    )?;
    validate_metadata_ranges_do_not_overlap(
        context,
        &[
            (
                "ktx2 data format descriptor",
                u64::from(dfd_byte_offset),
                u64::from(dfd_byte_length),
            ),
            (
                "ktx2 key/value data",
                u64::from(key_value_data_offset),
                u64::from(key_value_data_length),
            ),
            (
                "ktx2 supercompression global data",
                supercompression_global_data_offset,
                supercompression_global_data_length,
            ),
        ],
    )?;
    validate_metadata_ranges_do_not_overlap_level_payloads(
        context,
        &[
            (
                "ktx2 data format descriptor",
                u64::from(dfd_byte_offset),
                u64::from(dfd_byte_length),
            ),
            (
                "ktx2 key/value data",
                u64::from(key_value_data_offset),
                u64::from(key_value_data_length),
            ),
            (
                "ktx2 supercompression global data",
                supercompression_global_data_offset,
                supercompression_global_data_length,
            ),
        ],
        &level_payloads.ranges,
    )?;
    validate_key_value_data_offset(
        context,
        level_index_end,
        dfd_byte_offset,
        dfd_byte_length,
        key_value_data_offset,
        key_value_data_length,
    )?;
    validate_supercompression_global_data_offset(
        context,
        level_index_end,
        dfd_byte_offset,
        dfd_byte_length,
        key_value_data_offset,
        key_value_data_length,
        supercompression_global_data_offset,
        supercompression_global_data_length,
    )?;
    validate_key_value_data_records(context, key_value_data_offset, key_value_data_length)?;
    let dimension = texture_dimension_from_header(raw_height, raw_depth);
    let array_layers = texture_array_layers(dimension, parsed_layers);

    let upload_bytes =
        expand_standard_supercompressed_levels(context, level_count, supercompression)?;
    let upload_supercompression = if upload_bytes.is_some() {
        KTX2_SUPERCOMPRESSION_NONE
    } else {
        supercompression
    };

    Ok(TextureContainerInfo {
        format: format!("ktx2/vk-{vk_format}/supercompression-{upload_supercompression}"),
        upload_bytes,
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, raw_depth, array_layers),
        mip_count: level_count,
        array_layers,
    })
}

fn validate_supercompression_scheme(
    context: &AssetImportContext,
    scheme: u32,
) -> Result<(), AssetImportError> {
    match scheme {
        KTX2_SUPERCOMPRESSION_NONE
        | KTX2_SUPERCOMPRESSION_BASIS_LZ
        | KTX2_SUPERCOMPRESSION_ZSTANDARD
        | KTX2_SUPERCOMPRESSION_ZLIB => Ok(()),
        value => parse_error(
            context,
            format!("ktx2 supercompression scheme {value} is not supported by container importer"),
        ),
    }
}

fn validate_vk_format_supercompression_pair(
    context: &AssetImportContext,
    vk_format: u32,
    supercompression: u32,
) -> Result<(), AssetImportError> {
    if supercompression == KTX2_SUPERCOMPRESSION_BASIS_LZ && vk_format != 0 {
        return parse_error(
            context,
            format!("ktx2 BasisLZ supercompression requires vkFormat 0, got {vk_format}"),
        );
    }
    Ok(())
}

fn validate_supercompression_global_data(
    context: &AssetImportContext,
    supercompression: u32,
    length: u64,
    has_non_empty_level_payload: bool,
) -> Result<(), AssetImportError> {
    if supercompression == KTX2_SUPERCOMPRESSION_NONE && length != 0 {
        return parse_error(
            context,
            "ktx2 supercompression global data must be empty when supercompression is none",
        );
    }
    if matches!(
        supercompression,
        KTX2_SUPERCOMPRESSION_ZSTANDARD | KTX2_SUPERCOMPRESSION_ZLIB
    ) && length != 0
    {
        return parse_error(
            context,
            format!(
                "ktx2 supercompression global data must be empty for supercompression scheme {supercompression}"
            ),
        );
    }
    if supercompression == KTX2_SUPERCOMPRESSION_BASIS_LZ
        && has_non_empty_level_payload
        && length == 0
    {
        return parse_error(
            context,
            "ktx2 BasisLZ level payloads require supercompression global data",
        );
    }
    Ok(())
}

// The level-index scan feeds both metadata overlap checks and supercompression consistency checks.
struct LevelPayloadRanges {
    ranges: Vec<(u32, u64, u64)>,
    has_non_empty_payload: bool,
}

fn validate_level_payload_ranges(
    context: &AssetImportContext,
    level_count: u32,
    level_index_end: u64,
    supercompression: u32,
    image_count: u32,
) -> Result<LevelPayloadRanges, AssetImportError> {
    let mut occupied_ranges = Vec::new();
    let mut has_non_empty_payload = false;
    for level_index in 0..level_count {
        let entry_offset = level_entry_offset(context, level_index)?;
        let level_byte_offset = read_u64_le(context, entry_offset)?;
        let level_byte_length = read_u64_le(context, entry_offset + 8)?;
        let level_uncompressed_byte_length = read_u64_le(context, entry_offset + 16)?;
        validate_level_uncompressed_byte_length(
            context,
            level_index,
            level_byte_length,
            level_uncompressed_byte_length,
            supercompression,
            image_count,
        )?;
        if level_byte_length > 0 && level_byte_offset < level_index_end {
            return parse_error(
                context,
                format!("ktx2 level {level_index} payload starts inside header or level index"),
            );
        }
        if level_byte_length == 0 && level_byte_offset != 0 {
            return parse_error(
                context,
                format!("ktx2 level {level_index} payload offset must be 0 when byte length is 0"),
            );
        }
        if level_byte_length > 0 {
            validate_level_payload_alignment(context, level_index, level_byte_offset)?;
            has_non_empty_payload = true;
            reject_overlapping_level_range(
                context,
                level_index,
                level_byte_offset,
                level_byte_length,
                &occupied_ranges,
            )?;
            occupied_ranges.push((level_index, level_byte_offset, level_byte_length));
        }
        require_u64_range(
            context,
            level_byte_offset,
            level_byte_length,
            format!("ktx2 level {level_index} payload"),
        )?;
    }
    Ok(LevelPayloadRanges {
        ranges: occupied_ranges,
        has_non_empty_payload,
    })
}

fn validate_level_payload_alignment(
    context: &AssetImportContext,
    level_index: u32,
    level_byte_offset: u64,
) -> Result<(), AssetImportError> {
    if level_byte_offset % 8 != 0 {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} payload offset must be 8-byte aligned, got {level_byte_offset}"
            ),
        );
    }
    Ok(())
}

fn reject_overlapping_level_range(
    context: &AssetImportContext,
    level_index: u32,
    offset: u64,
    length: u64,
    occupied_ranges: &[(u32, u64, u64)],
) -> Result<(), AssetImportError> {
    let end = checked_u64_range_end(context, "ktx2 level payload", offset, length)?;
    for (occupied_level, occupied_offset, occupied_length) in occupied_ranges {
        let occupied_end = checked_u64_range_end(
            context,
            "ktx2 level payload",
            *occupied_offset,
            *occupied_length,
        )?;
        if offset < occupied_end && *occupied_offset < end {
            return parse_error(
                context,
                format!("ktx2 level {level_index} payload overlaps level {occupied_level} payload"),
            );
        }
    }
    Ok(())
}

fn validate_metadata_ranges_do_not_overlap(
    context: &AssetImportContext,
    ranges: &[(&str, u64, u64)],
) -> Result<(), AssetImportError> {
    for (range_index, (label, offset, length)) in ranges.iter().copied().enumerate() {
        if length == 0 {
            continue;
        }
        let end = checked_u64_range_end(context, label, offset, length)?;
        for (occupied_label, occupied_offset, occupied_length) in
            ranges[..range_index].iter().copied()
        {
            if occupied_length == 0 {
                continue;
            }
            let occupied_end =
                checked_u64_range_end(context, occupied_label, occupied_offset, occupied_length)?;
            if offset < occupied_end && occupied_offset < end {
                return parse_error(context, format!("{label} overlaps {occupied_label}"));
            }
        }
    }
    Ok(())
}

fn validate_metadata_ranges_do_not_overlap_level_payloads(
    context: &AssetImportContext,
    metadata_ranges: &[(&str, u64, u64)],
    level_payload_ranges: &[(u32, u64, u64)],
) -> Result<(), AssetImportError> {
    for (label, offset, length) in metadata_ranges.iter().copied() {
        if length == 0 {
            continue;
        }
        let end = checked_u64_range_end(context, label, offset, length)?;
        for (level_index, level_offset, level_length) in level_payload_ranges {
            let level_end =
                checked_u64_range_end(context, "ktx2 level payload", *level_offset, *level_length)?;
            if offset < level_end && *level_offset < end {
                return parse_error(
                    context,
                    format!("{label} overlaps ktx2 level {level_index} payload"),
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod plugins07_metadata_hotpath_tests {
    use std::{hint::black_box, time::Instant};

    use super::*;
    use zircon_runtime::asset::AssetUri;

    const RANGES: [(&str, u64, u64); 3] = [
        ("ktx2 data format descriptor", 128, 32),
        ("ktx2 key/value data", 192, 64),
        ("ktx2 supercompression global data", 320, 96),
    ];

    fn context() -> AssetImportContext {
        AssetImportContext::new(
            "metadata-hotpath.ktx2".into(),
            AssetUri::parse("res://textures/metadata-hotpath.ktx2").expect("valid asset URI"),
            Vec::new(),
            "".parse().expect("valid default texture settings"),
        )
    }

    fn legacy_validate_metadata_ranges_do_not_overlap(
        context: &AssetImportContext,
        ranges: &[(&str, u64, u64)],
    ) -> Result<(), AssetImportError> {
        let mut occupied_ranges: Vec<(&str, u64, u64)> = Vec::new();
        for (label, offset, length) in ranges.iter().copied() {
            if length == 0 {
                continue;
            }
            let end = checked_u64_range_end(context, label, offset, length)?;
            for (occupied_label, occupied_offset, occupied_length) in &occupied_ranges {
                let occupied_end = checked_u64_range_end(
                    context,
                    occupied_label,
                    *occupied_offset,
                    *occupied_length,
                )?;
                if offset < occupied_end && *occupied_offset < end {
                    return parse_error(context, format!("{label} overlaps {occupied_label}"));
                }
            }
            occupied_ranges.push((label, offset, length));
        }
        Ok(())
    }

    #[test]
    fn plugins07_container_hotpath_metadata_overlap_preserves_fail_closed_behavior() {
        let context = context();
        validate_metadata_ranges_do_not_overlap(&context, &RANGES)
            .expect("disjoint metadata ranges should pass");
        validate_metadata_ranges_do_not_overlap(
            &context,
            &[("empty", 0, 0), ("occupied", 128, 32)],
        )
        .expect("empty metadata ranges should not occupy an interval");

        let error = validate_metadata_ranges_do_not_overlap(
            &context,
            &[("first", 128, 32), ("second", 144, 32)],
        )
        .expect_err("overlapping metadata ranges must fail closed")
        .to_string();
        assert!(
            error.contains("second overlaps first"),
            "unexpected error: {error}"
        );
    }

    #[test]
    #[ignore = "release-only KTX2 metadata overlap benchmark"]
    fn plugins07_container_hotpath_release_allocation_free_metadata_overlap_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 50_000;

        fn measure_legacy(context: &AssetImportContext) -> u128 {
            let started = Instant::now();
            for _ in 0..CHECKS_PER_SAMPLE {
                black_box(legacy_validate_metadata_ranges_do_not_overlap(
                    black_box(context),
                    black_box(&RANGES),
                ))
                .expect("benchmark ranges stay disjoint");
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(context: &AssetImportContext) -> u128 {
            let started = Instant::now();
            for _ in 0..CHECKS_PER_SAMPLE {
                black_box(validate_metadata_ranges_do_not_overlap(
                    black_box(context),
                    black_box(&RANGES),
                ))
                .expect("benchmark ranges stay disjoint");
            }
            started.elapsed().as_nanos().max(1)
        }

        let context = context();
        for _ in 0..4 {
            black_box(measure_legacy(&context));
            black_box(measure_optimized(&context));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&context));
                optimized_samples.push(measure_optimized(&context));
            } else {
                optimized_samples.push(measure_optimized(&context));
                legacy_samples.push(measure_legacy(&context));
            }
        }

        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
        println!(
            "PERF_RESULT plugins07_ktx2_metadata_overlap sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} range_count={} order=alternating_legacy_first_even \
legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_vec_instances_per_sample={CHECKS_PER_SAMPLE} optimized_vec_instances_per_sample=0 \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
improvement_percent={improvement_percent} threshold_percent=25 \
legacy_ns={} optimized_ns={}",
            RANGES.len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
            "allocation-free metadata overlap validation must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
        if optimized >= legacy {
            0
        } else {
            legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
        }
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn validate_level_uncompressed_byte_length(
    context: &AssetImportContext,
    level_index: u32,
    byte_length: u64,
    uncompressed_byte_length: u64,
    supercompression: u32,
    image_count: u32,
) -> Result<(), AssetImportError> {
    if byte_length == 0 && uncompressed_byte_length != 0 {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} uncompressed byte length must be 0 when byte length is 0"
            ),
        );
    }
    if supercompression == KTX2_SUPERCOMPRESSION_NONE && uncompressed_byte_length != byte_length {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} uncompressed byte length must equal byte length when supercompression is none"
            ),
        );
    }
    if supercompression == KTX2_SUPERCOMPRESSION_BASIS_LZ && uncompressed_byte_length != 0 {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} uncompressed byte length must be 0 for BasisLZ supercompression"
            ),
        );
    }
    if matches!(
        supercompression,
        KTX2_SUPERCOMPRESSION_ZSTANDARD | KTX2_SUPERCOMPRESSION_ZLIB
    ) && byte_length > 0
        && uncompressed_byte_length == 0
    {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} uncompressed byte length must be nonzero for supercompression scheme {supercompression} when byte length is nonzero"
            ),
        );
    }
    if matches!(
        supercompression,
        KTX2_SUPERCOMPRESSION_ZSTANDARD | KTX2_SUPERCOMPRESSION_ZLIB
    ) && uncompressed_byte_length < byte_length
    {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} uncompressed byte length must be at least byte length for supercompression scheme {supercompression}"
            ),
        );
    }
    if uncompressed_byte_length % u64::from(image_count) != 0 {
        return parse_error(
            context,
            format!(
                "ktx2 level {level_index} uncompressed byte length must be divisible by image count {image_count}, got {uncompressed_byte_length}"
            ),
        );
    }
    Ok(())
}

fn validate_key_value_data_records(
    context: &AssetImportContext,
    key_value_data_offset: u32,
    key_value_data_length: u32,
) -> Result<(), AssetImportError> {
    if key_value_data_length == 0 {
        return Ok(());
    }

    let metadata_start = usize::try_from(key_value_data_offset)
        .map_err(|_| parse_error_value(context, "ktx2 key/value data offset overflows usize"))?;
    let metadata_len = usize::try_from(key_value_data_length)
        .map_err(|_| parse_error_value(context, "ktx2 key/value data length overflows usize"))?;
    let metadata_end = metadata_start
        .checked_add(metadata_len)
        .ok_or_else(|| parse_error_value(context, "ktx2 key/value data range overflows usize"))?;
    let mut cursor = metadata_start;
    let mut record_index = 0_usize;
    while cursor < metadata_end {
        let length_label =
            format!("ktx2 key/value data record {record_index} keyAndValueByteLength");
        let size_end = cursor
            .checked_add(4)
            .ok_or_else(|| parse_error_value(context, format!("{length_label} overflows usize")))?;
        require_key_value_data_record_range(context, size_end, metadata_end, &length_label)?;
        let key_and_value_len = usize::try_from(read_u32_le(context, cursor)?)
            .map_err(|_| parse_error_value(context, format!("{length_label} overflows usize")))?;
        if key_and_value_len < 2 {
            return parse_error(context, format!("{length_label} must be at least 2 bytes"));
        }

        let payload_label = format!("ktx2 key/value data record {record_index} payload");
        let payload_end = size_end.checked_add(key_and_value_len).ok_or_else(|| {
            parse_error_value(context, format!("{payload_label} range overflows usize"))
        })?;
        require_key_value_data_record_range(context, payload_end, metadata_end, &payload_label)?;
        validate_key_value_data_key(context, record_index, size_end, payload_end)?;

        let padding_len = ktx_four_byte_padding(key_and_value_len);
        let padded_record_end = payload_end.checked_add(padding_len).ok_or_else(|| {
            parse_error_value(
                context,
                format!("{payload_label} padded range overflows usize"),
            )
        })?;
        require_key_value_data_record_range(
            context,
            padded_record_end,
            metadata_end,
            &format!("ktx2 key/value data record {record_index} valuePadding"),
        )?;
        if context.source_bytes[payload_end..padded_record_end]
            .iter()
            .any(|byte| *byte != 0)
        {
            return parse_error(
                context,
                format!(
                    "ktx2 key/value data record {record_index} valuePadding bytes must be zero"
                ),
            );
        }

        cursor = padded_record_end;
        record_index += 1;
    }
    Ok(())
}

fn validate_key_value_data_offset(
    context: &AssetImportContext,
    level_index_end: u64,
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
    key_value_data_offset: u32,
    key_value_data_length: u32,
) -> Result<(), AssetImportError> {
    if key_value_data_length == 0 {
        return Ok(());
    }
    let (pre_kvd_end, preceding_label) = if dfd_byte_length > 0 {
        (
            u64::from(dfd_byte_offset)
                .checked_add(u64::from(dfd_byte_length))
                .ok_or_else(|| {
                    parse_error_value(context, "ktx2 data format descriptor range overflows u64")
                })?,
            "data format descriptor",
        )
    } else {
        (level_index_end, "level index")
    };
    let expected_kvd_offset = align_to_four(pre_kvd_end).ok_or_else(|| {
        parse_error_value(context, "ktx2 key/value data aligned offset overflows u64")
    })?;
    if u64::from(key_value_data_offset) != expected_kvd_offset {
        return parse_error(
            context,
            format!(
                "ktx2 key/value data offset must be {expected_kvd_offset} after {preceding_label} alignment, got {key_value_data_offset}"
            ),
        );
    }
    Ok(())
}

fn require_key_value_data_record_range(
    context: &AssetImportContext,
    required: usize,
    metadata_end: usize,
    label: &str,
) -> Result<(), AssetImportError> {
    if required > metadata_end {
        return parse_error(
            context,
            format!("{label} extends past declared ktx2 key/value data length"),
        );
    }
    Ok(())
}

fn validate_key_value_data_key(
    context: &AssetImportContext,
    record_index: usize,
    key_and_value_start: usize,
    key_and_value_end: usize,
) -> Result<(), AssetImportError> {
    let key_and_value = &context.source_bytes[key_and_value_start..key_and_value_end];
    let Some(nul_index) = key_and_value.iter().position(|byte| *byte == 0) else {
        return parse_error(
            context,
            format!("ktx2 key/value data record {record_index} key must be NUL terminated"),
        );
    };
    if nul_index == 0 {
        return parse_error(
            context,
            format!("ktx2 key/value data record {record_index} key must be non-empty"),
        );
    }
    let key = &key_and_value[..nul_index];
    if key.starts_with(&[0xef, 0xbb, 0xbf]) || std::str::from_utf8(key).is_err() {
        return parse_error(
            context,
            format!("ktx2 key/value data record {record_index} key must be UTF-8 without BOM"),
        );
    }
    Ok(())
}

fn validate_zero_length_metadata_offset(
    context: &AssetImportContext,
    offset: u64,
    length: u64,
    label: &str,
) -> Result<(), AssetImportError> {
    if length == 0 && offset != 0 {
        return parse_error(
            context,
            format!("{label} offset must be 0 when length is 0"),
        );
    }
    Ok(())
}

fn validate_supercompression_global_data_offset(
    context: &AssetImportContext,
    level_index_end: u64,
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
    key_value_data_offset: u32,
    key_value_data_length: u32,
    supercompression_global_data_offset: u64,
    supercompression_global_data_length: u64,
) -> Result<(), AssetImportError> {
    if supercompression_global_data_length == 0 {
        return Ok(());
    }
    let pre_sgd_end = if key_value_data_length > 0 {
        u64::from(key_value_data_offset)
            .checked_add(u64::from(key_value_data_length))
            .ok_or_else(|| parse_error_value(context, "ktx2 key/value data range overflows u64"))?
    } else if dfd_byte_length > 0 {
        u64::from(dfd_byte_offset)
            .checked_add(u64::from(dfd_byte_length))
            .ok_or_else(|| {
                parse_error_value(context, "ktx2 data format descriptor range overflows u64")
            })?
    } else {
        level_index_end
    };
    let expected_sgd_offset = align_to_eight(pre_sgd_end).ok_or_else(|| {
        parse_error_value(
            context,
            "ktx2 supercompression global data aligned offset overflows u64",
        )
    })?;
    if supercompression_global_data_offset != expected_sgd_offset {
        return parse_error(
            context,
            format!(
                "ktx2 supercompression global data offset must be {expected_sgd_offset} after key/value data alignment, got {supercompression_global_data_offset}"
            ),
        );
    }
    Ok(())
}

fn align_to_four(value: u64) -> Option<u64> {
    value.checked_add(3).map(|value| value & !3)
}

fn align_to_eight(value: u64) -> Option<u64> {
    value.checked_add(7).map(|value| value & !7)
}

fn require_metadata_range(
    context: &AssetImportContext,
    offset: u64,
    length: u64,
    level_index_end: u64,
    label: &str,
) -> Result<(), AssetImportError> {
    if length > 0 && offset < level_index_end {
        return parse_error(
            context,
            format!("{label} starts inside header or level index"),
        );
    }
    require_u64_range(context, offset, length, label)
}

fn require_u64_range(
    context: &AssetImportContext,
    offset: u64,
    length: u64,
    label: impl AsRef<str>,
) -> Result<(), AssetImportError> {
    let label = label.as_ref();
    let end = usize::try_from(offset)
        .ok()
        .and_then(|offset| {
            usize::try_from(length)
                .ok()
                .and_then(|length| offset.checked_add(length))
        })
        .ok_or_else(|| parse_error_value(context, format!("{label} range overflows usize")))?;
    require_len(context, end, label)
}
