use super::{
    ktx_four_byte_padding, read_face_count, texture_array_layers, texture_dimension_from_header,
    validate_3d_array_layers, validate_3d_height, validate_cubemap_2d_square_faces,
    validate_cubemap_depth, validate_mip_count_fits_extent, TextureContainerInfo,
};
use crate::container::support::{
    checked_layer_count, parse_error, parse_error_value, read_nonzero_u32, read_u32_le,
    require_len, texture_depth_or_array_layers, KTX1_IDENTIFIER, KTX_LITTLE_ENDIAN,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};

pub(super) fn parse(
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

    let gl_type = read_u32_le(context, 16)?;
    let gl_type_size = read_nonzero_u32(context, 20, "ktx glTypeSize")?;
    let gl_format = read_u32_le(context, 24)?;
    let gl_internal_format = read_nonzero_u32(context, 28, "ktx glInternalFormat")?;
    validate_type_format_pair(
        context,
        gl_type,
        gl_type_size,
        gl_format,
        gl_internal_format,
    )?;
    let _gl_base_internal_format = read_nonzero_u32(context, 32, "ktx glBaseInternalFormat")?;
    let width = read_nonzero_u32(context, 36, "ktx width")?;
    let raw_height = read_u32_le(context, 40)?;
    let raw_depth = read_u32_le(context, 44)?;
    validate_3d_height(context, "ktx", raw_height, raw_depth)?;
    let height = raw_height.max(1);
    let raw_array_elements = read_u32_le(context, 48)?;
    validate_3d_array_layers(context, "ktx", raw_array_elements, raw_depth)?;
    let array_elements = raw_array_elements.max(1);
    let faces = read_face_count(context, 52, "ktx face count")?;
    validate_cubemap_depth(context, "ktx", faces, raw_depth)?;
    validate_cubemap_2d_square_faces(context, "ktx", faces, width, raw_height)?;
    let mip_count = read_u32_le(context, 56)?.max(1);
    validate_mip_count_fits_extent(context, "ktx", width, height, raw_depth.max(1), mip_count)?;
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
    validate_key_value_metadata_records(context, 64, required_metadata_len)?;
    validate_level_ranges(context, required_metadata_len, mip_count)?;
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

fn validate_type_format_pair(
    context: &AssetImportContext,
    gl_type: u32,
    gl_type_size: u32,
    gl_format: u32,
    gl_internal_format: u32,
) -> Result<(), AssetImportError> {
    if !matches!(gl_type_size, 1 | 2 | 4) {
        return parse_error(context, "ktx glTypeSize must be 1, 2, or 4 bytes");
    }
    if (gl_type == 0) != (gl_format == 0) {
        return parse_error(
            context,
            "ktx glType and glFormat must both be zero for compressed data or both be nonzero for uncompressed data",
        );
    }
    if gl_type == 0 && gl_type_size != 1 {
        return parse_error(context, "ktx glTypeSize must be 1 for compressed data");
    }
    if gl_format == gl_internal_format {
        return parse_error(context, "ktx glInternalFormat must not equal glFormat");
    }
    Ok(())
}

fn validate_key_value_metadata_records(
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
        require_metadata_record_range(context, size_end, metadata_end, &size_label)?;
        let key_and_value_len = usize::try_from(read_u32_le(context, cursor)?)
            .map_err(|_| parse_error_value(context, format!("{size_label} overflows usize")))?;
        if key_and_value_len == 0 {
            return parse_error(context, format!("{size_label} must be nonzero"));
        }

        let payload_label = format!("ktx key/value metadata record {record_index} payload");
        let payload_end = size_end.checked_add(key_and_value_len).ok_or_else(|| {
            parse_error_value(context, format!("{payload_label} range overflows usize"))
        })?;
        require_metadata_record_range(context, payload_end, metadata_end, &payload_label)?;
        validate_key_value_metadata_key(context, record_index, size_end, payload_end)?;
        let padded_record_end = payload_end
            .checked_add(ktx_four_byte_padding(key_and_value_len))
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    format!("{payload_label} padded range overflows usize"),
                )
            })?;
        require_metadata_record_range(
            context,
            padded_record_end,
            metadata_end,
            &format!("{payload_label} padding"),
        )?;
        if context.source_bytes[payload_end..padded_record_end]
            .iter()
            .any(|byte| *byte != 0)
        {
            return parse_error(
                context,
                format!("ktx key/value metadata record {record_index} padding bytes must be zero"),
            );
        }
        cursor = padded_record_end;
        record_index += 1;
    }
    Ok(())
}

fn validate_key_value_metadata_key(
    context: &AssetImportContext,
    record_index: usize,
    key_and_value_start: usize,
    key_and_value_end: usize,
) -> Result<(), AssetImportError> {
    let key_and_value = &context.source_bytes[key_and_value_start..key_and_value_end];
    let Some(nul_index) = key_and_value.iter().position(|byte| *byte == 0) else {
        return parse_error(
            context,
            format!("ktx key/value metadata record {record_index} key must be NUL terminated"),
        );
    };
    if nul_index == 0 {
        return parse_error(
            context,
            format!("ktx key/value metadata record {record_index} key must be non-empty"),
        );
    }
    let key = &key_and_value[..nul_index];
    if key.starts_with(&[0xef, 0xbb, 0xbf]) || std::str::from_utf8(key).is_err() {
        return parse_error(
            context,
            format!("ktx key/value metadata record {record_index} key must be UTF-8 without BOM"),
        );
    }
    Ok(())
}

fn require_metadata_record_range(
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

fn validate_level_ranges(
    context: &AssetImportContext,
    first_level_offset: usize,
    mip_count: u32,
) -> Result<(), AssetImportError> {
    let mut cursor = first_level_offset;
    for level_index in 0..mip_count {
        // KTX1 stores each mip level as imageSize, payload bytes, then 4-byte padding.
        let image_size_label = level_image_size_label(level_index);
        let image_size_end = cursor.checked_add(4).ok_or_else(|| {
            parse_error_value(context, format!("{image_size_label} overflows usize"))
        })?;
        require_len(context, image_size_end, &image_size_label)?;
        let image_size = usize::try_from(read_u32_le(context, cursor)?).map_err(|_| {
            parse_error_value(context, format!("{image_size_label} overflows usize"))
        })?;

        let payload_label = level_payload_label(level_index);
        let payload_end = image_size_end.checked_add(image_size).ok_or_else(|| {
            parse_error_value(context, format!("{payload_label} range overflows usize"))
        })?;
        if image_size > 0 {
            require_len(context, payload_end, &payload_label)?;
        }
        cursor = payload_end;

        if level_index + 1 < mip_count {
            let padding_len = ktx_four_byte_padding(image_size);
            let padded_payload_end = cursor.checked_add(padding_len).ok_or_else(|| {
                parse_error_value(
                    context,
                    format!("{payload_label} padded range overflows usize"),
                )
            })?;
            require_len(
                context,
                padded_payload_end,
                &format!("{payload_label} padding"),
            )?;
            if context.source_bytes[cursor..padded_payload_end]
                .iter()
                .any(|byte| *byte != 0)
            {
                return parse_error(
                    context,
                    format!("{payload_label} padding bytes must be zero"),
                );
            }
            cursor = padded_payload_end;
        }
    }
    Ok(())
}

fn level_image_size_label(level_index: u32) -> String {
    if level_index == 0 {
        "ktx first mip level imageSize".to_string()
    } else {
        format!("ktx mip level {level_index} imageSize")
    }
}

fn level_payload_label(level_index: u32) -> String {
    if level_index == 0 {
        "ktx first mip level payload".to_string()
    } else {
        format!("ktx mip level {level_index} payload")
    }
}
