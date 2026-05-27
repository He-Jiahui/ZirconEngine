use super::{
    support::{
        checked_layer_count, parse_error, parse_error_value, read_nonzero_u32, read_u32_le,
        read_u64_le, require_len, texture_depth_or_array_layers, KTX1_IDENTIFIER, KTX2_HEADER_SIZE,
        KTX2_IDENTIFIER, KTX2_LEVEL_INDEX_ENTRY_SIZE, KTX2_SUPERCOMPRESSION_BASIS_LZ,
        KTX2_SUPERCOMPRESSION_NONE, KTX2_SUPERCOMPRESSION_ZLIB, KTX2_SUPERCOMPRESSION_ZSTANDARD,
        KTX_LITTLE_ENDIAN,
    },
    TextureContainerInfo,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

pub(super) fn parse_ktx1(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, 64, "ktx header")?;
    if &bytes[..12] != KTX1_IDENTIFIER {
        return parse_error(context, "ktx header missing KTX 1 identifier");
    }
    if read_u32_le(context, 12)? != KTX_LITTLE_ENDIAN {
        return parse_error(context, "only little-endian KTX 1 files are supported");
    }

    let gl_internal_format = read_u32_le(context, 28)?;
    let width = read_nonzero_u32(context, 36, "ktx width")?;
    let raw_height = read_u32_le(context, 40)?;
    let raw_depth = read_u32_le(context, 44)?;
    validate_3d_height(context, "ktx", raw_height, raw_depth)?;
    let height = raw_height.max(1);
    let array_elements = read_u32_le(context, 48)?.max(1);
    let faces = read_face_count(context, 52, "ktx face count")?;
    let mip_count = read_u32_le(context, 56)?.max(1);
    validate_ktx_mip_count_fits_extent(context, "ktx", width, height, raw_depth.max(1), mip_count)?;
    let metadata_len = usize::try_from(read_u32_le(context, 60)?)
        .map_err(|_| parse_error_value(context, "ktx key/value metadata length overflows usize"))?;
    if metadata_len % 4 != 0 {
        return parse_error(
            context,
            "ktx key/value metadata length must be a multiple of 4 bytes",
        );
    }
    let required_metadata_len = 64usize.checked_add(metadata_len).ok_or_else(|| {
        parse_error_value(context, "ktx key/value metadata length overflows usize")
    })?;
    require_len(context, required_metadata_len, "ktx key/value metadata")?;
    validate_ktx1_key_value_metadata_records(context, 64, required_metadata_len)?;
    validate_ktx1_level_ranges(context, required_metadata_len, mip_count)?;
    let dimension = texture_dimension_from_header(raw_height, raw_depth);
    let parsed_layers =
        checked_layer_count(context, "ktx array layer count", array_elements, faces)?;
    let array_layers = texture_array_layers(dimension, parsed_layers);

    Ok(TextureContainerInfo {
        format: format!("ktx/gl-internal-0x{gl_internal_format:08x}"),
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, raw_depth, array_layers),
        mip_count,
        array_layers,
    })
}

pub(super) fn parse_ktx2(
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
    let layer_count = read_u32_le(context, 32)?.max(1);
    let face_count = read_face_count(context, 36, "ktx2 face count")?;
    let level_count = read_u32_le(context, 40)?.max(1);
    validate_ktx_mip_count_fits_extent(
        context,
        "ktx2",
        width,
        height,
        raw_depth.max(1),
        level_count,
    )?;
    let supercompression = read_u32_le(context, 44)?;
    validate_supercompression_scheme(context, supercompression)?;
    let required_level_index_len = usize::try_from(level_count)
        .ok()
        .and_then(|count| count.checked_mul(KTX2_LEVEL_INDEX_ENTRY_SIZE))
        .and_then(|level_index_len| KTX2_HEADER_SIZE.checked_add(level_index_len))
        .ok_or_else(|| parse_error_value(context, "ktx2 level index length overflows usize"))?;
    require_len(context, required_level_index_len, "ktx2 level index")?;
    let level_index_end = level_index_end(context, level_count)?;
    let level_payload_ranges = validate_level_payload_ranges(
        context,
        level_count,
        level_index_end,
        supercompression,
        layer_count,
        face_count,
    )?;
    let dfd_byte_offset = read_u32_le(context, 48)?;
    let dfd_byte_length = read_u32_le(context, 52)?;
    require_metadata_range(
        context,
        u64::from(dfd_byte_offset),
        u64::from(dfd_byte_length),
        level_index_end,
        "ktx2 data format descriptor",
    )?;
    validate_data_format_descriptor(context, dfd_byte_offset, dfd_byte_length)?;
    let key_value_data_offset = read_u32_le(context, 56)?;
    let key_value_data_length = read_u32_le(context, 60)?;
    require_metadata_range(
        context,
        u64::from(key_value_data_offset),
        u64::from(key_value_data_length),
        level_index_end,
        "ktx2 key/value data",
    )?;
    let supercompression_global_data_offset = read_u64_le(context, 64)?;
    let supercompression_global_data_length = read_u64_le(context, 72)?;
    require_metadata_range(
        context,
        supercompression_global_data_offset,
        supercompression_global_data_length,
        level_index_end,
        "ktx2 supercompression global data",
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
        &level_payload_ranges,
    )?;
    let dimension = texture_dimension_from_header(raw_height, raw_depth);
    let parsed_layers =
        checked_layer_count(context, "ktx2 array layer count", layer_count, face_count)?;
    let array_layers = texture_array_layers(dimension, parsed_layers);

    Ok(TextureContainerInfo {
        format: format!("ktx2/vk-{vk_format}/supercompression-{supercompression}"),
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, raw_depth, array_layers),
        mip_count: level_count,
        array_layers,
    })
}

fn texture_dimension_from_header(height: u32, depth: u32) -> RenderImageDimension {
    if depth > 0 {
        RenderImageDimension::D3
    } else if height == 0 {
        RenderImageDimension::D1
    } else {
        RenderImageDimension::D2
    }
}

fn validate_3d_height(
    context: &AssetImportContext,
    label: &str,
    height: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && height == 0 {
        return parse_error(
            context,
            format!("{label} 3d texture height must be nonzero when depth is nonzero"),
        );
    }
    Ok(())
}

fn texture_array_layers(dimension: RenderImageDimension, array_layers: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        array_layers.max(1)
    }
}

fn validate_ktx_mip_count_fits_extent(
    context: &AssetImportContext,
    label: &str,
    width: u32,
    height: u32,
    depth: u32,
    mip_count: u32,
) -> Result<(), AssetImportError> {
    let max_extent = width.max(height).max(depth);
    let max_mip_count = u32::BITS - max_extent.leading_zeros();
    if mip_count > max_mip_count {
        return parse_error(
            context,
            format!(
                "{label} mip level count {mip_count} exceeds maximum {max_mip_count} for extent {width}x{height}x{depth}"
            ),
        );
    }
    Ok(())
}

fn read_face_count(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u32, AssetImportError> {
    match read_u32_le(context, offset)? {
        0 | 1 => Ok(1),
        6 => Ok(6),
        value => parse_error(
            context,
            format!("{label} must be 1 for ordinary textures or 6 for cubemaps, got {value}"),
        ),
    }
}

fn validate_ktx1_key_value_metadata_records(
    context: &AssetImportContext,
    metadata_start: usize,
    metadata_end: usize,
) -> Result<(), AssetImportError> {
    let mut cursor = metadata_start;
    let mut record_index = 0_usize;
    while cursor < metadata_end {
        let size_label =
            format!("ktx key/value metadata record {record_index} keyAndValueByteSize");
        let size_end = cursor
            .checked_add(4)
            .ok_or_else(|| parse_error_value(context, format!("{size_label} overflows usize")))?;
        require_ktx1_metadata_record_range(context, size_end, metadata_end, &size_label)?;
        let key_and_value_len = usize::try_from(read_u32_le(context, cursor)?)
            .map_err(|_| parse_error_value(context, format!("{size_label} overflows usize")))?;
        if key_and_value_len == 0 {
            return parse_error(context, format!("{size_label} must be nonzero"));
        }

        let payload_label = format!("ktx key/value metadata record {record_index} payload");
        let payload_end = size_end.checked_add(key_and_value_len).ok_or_else(|| {
            parse_error_value(context, format!("{payload_label} range overflows usize"))
        })?;
        require_ktx1_metadata_record_range(context, payload_end, metadata_end, &payload_label)?;
        let padded_record_end = payload_end
            .checked_add(ktx1_padding(key_and_value_len))
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    format!("{payload_label} padded range overflows usize"),
                )
            })?;
        require_ktx1_metadata_record_range(
            context,
            padded_record_end,
            metadata_end,
            &format!("{payload_label} padding"),
        )?;
        cursor = padded_record_end;
        record_index += 1;
    }
    Ok(())
}

fn require_ktx1_metadata_record_range(
    context: &AssetImportContext,
    required: usize,
    metadata_end: usize,
    label: &str,
) -> Result<(), AssetImportError> {
    if required > metadata_end {
        return parse_error(
            context,
            format!("{label} extends past declared ktx key/value metadata length"),
        );
    }
    Ok(())
}

fn validate_ktx1_level_ranges(
    context: &AssetImportContext,
    first_level_offset: usize,
    mip_count: u32,
) -> Result<(), AssetImportError> {
    let mut cursor = first_level_offset;
    for level_index in 0..mip_count {
        // KTX1 stores each mip level as imageSize, payload bytes, then 4-byte padding.
        let image_size_label = ktx1_level_image_size_label(level_index);
        let image_size_end = cursor.checked_add(4).ok_or_else(|| {
            parse_error_value(context, format!("{image_size_label} overflows usize"))
        })?;
        require_len(context, image_size_end, &image_size_label)?;
        let image_size = usize::try_from(read_u32_le(context, cursor)?).map_err(|_| {
            parse_error_value(context, format!("{image_size_label} overflows usize"))
        })?;

        let payload_label = ktx1_level_payload_label(level_index);
        let payload_end = image_size_end.checked_add(image_size).ok_or_else(|| {
            parse_error_value(context, format!("{payload_label} range overflows usize"))
        })?;
        if image_size > 0 {
            require_len(context, payload_end, &payload_label)?;
        }
        cursor = payload_end;

        if level_index + 1 < mip_count {
            cursor = cursor
                .checked_add(ktx1_padding(image_size))
                .ok_or_else(|| {
                    parse_error_value(
                        context,
                        format!("{payload_label} padded range overflows usize"),
                    )
                })?;
        }
    }
    Ok(())
}

fn ktx1_level_image_size_label(level_index: u32) -> String {
    if level_index == 0 {
        "ktx first mip level imageSize".to_string()
    } else {
        format!("ktx mip level {level_index} imageSize")
    }
}

fn ktx1_level_payload_label(level_index: u32) -> String {
    if level_index == 0 {
        "ktx first mip level payload".to_string()
    } else {
        format!("ktx mip level {level_index} payload")
    }
}

fn ktx1_padding(byte_len: usize) -> usize {
    (4 - (byte_len % 4)) % 4
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

fn validate_level_payload_ranges(
    context: &AssetImportContext,
    level_count: u32,
    level_index_end: u64,
    supercompression: u32,
    layer_count: u32,
    face_count: u32,
) -> Result<Vec<(u32, u64, u64)>, AssetImportError> {
    let image_count = checked_layer_count(context, "ktx2 image count", layer_count, face_count)?;
    let mut occupied_ranges = Vec::new();
    for level_index in 0..level_count {
        let entry_offset = KTX2_HEADER_SIZE
            .checked_add(
                usize::try_from(level_index)
                    .expect("u32 level index always fits usize")
                    .checked_mul(KTX2_LEVEL_INDEX_ENTRY_SIZE)
                    .ok_or_else(|| {
                        parse_error_value(context, "ktx2 level index entry offset overflows usize")
                    })?,
            )
            .ok_or_else(|| {
                parse_error_value(context, "ktx2 level index entry offset overflows usize")
            })?;
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
        if level_byte_length > 0 {
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
    Ok(occupied_ranges)
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
    let mut occupied_ranges = Vec::new();
    for (label, offset, length) in ranges.iter().copied() {
        if length == 0 {
            continue;
        }
        let end = checked_u64_range_end(context, label, offset, length)?;
        for (occupied_label, occupied_offset, occupied_length) in &occupied_ranges {
            let occupied_end =
                checked_u64_range_end(context, occupied_label, *occupied_offset, *occupied_length)?;
            if offset < occupied_end && *occupied_offset < end {
                return parse_error(context, format!("{label} overlaps {occupied_label}"));
            }
        }
        occupied_ranges.push((label, offset, length));
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

fn checked_u64_range_end(
    context: &AssetImportContext,
    label: &str,
    offset: u64,
    length: u64,
) -> Result<u64, AssetImportError> {
    offset
        .checked_add(length)
        .ok_or_else(|| parse_error_value(context, format!("{label} range overflows u64")))
}

fn validate_level_uncompressed_byte_length(
    context: &AssetImportContext,
    level_index: u32,
    byte_length: u64,
    uncompressed_byte_length: u64,
    supercompression: u32,
    image_count: u32,
) -> Result<(), AssetImportError> {
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

fn validate_data_format_descriptor(
    context: &AssetImportContext,
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
) -> Result<(), AssetImportError> {
    if dfd_byte_length == 0 {
        return Ok(());
    }
    if dfd_byte_length < 4 {
        return parse_error(
            context,
            "ktx2 data format descriptor length must be at least 4 bytes when present",
        );
    }
    let dfd_total_size = read_u32_le(
        context,
        usize::try_from(dfd_byte_offset).map_err(|_| {
            parse_error_value(
                context,
                "ktx2 data format descriptor offset overflows usize",
            )
        })?,
    )?;
    if dfd_total_size != dfd_byte_length {
        return parse_error(
            context,
            format!(
                "ktx2 data format descriptor total size {dfd_total_size} must equal dfdByteLength {dfd_byte_length}"
            ),
        );
    }
    Ok(())
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

fn level_index_end(
    context: &AssetImportContext,
    level_count: u32,
) -> Result<u64, AssetImportError> {
    u64::try_from(KTX2_HEADER_SIZE)
        .ok()
        .and_then(|header_size| {
            u64::from(level_count)
                .checked_mul(u64::try_from(KTX2_LEVEL_INDEX_ENTRY_SIZE).ok()?)
                .and_then(|level_index_len| header_size.checked_add(level_index_len))
        })
        .ok_or_else(|| parse_error_value(context, "ktx2 level index length overflows u64"))
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
