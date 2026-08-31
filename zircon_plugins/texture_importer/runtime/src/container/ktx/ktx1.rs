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
        upload_bytes: None,
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
        let size_end = cursor.checked_add(4).ok_or_else(|| {
            parse_error_value(
                context,
                format!(
                    "ktx key/value metadata record {record_index} keyAndValueByteSize overflows usize"
                ),
            )
        })?;
        require_metadata_record_range(context, size_end, metadata_end, || {
            format!("ktx key/value metadata record {record_index} keyAndValueByteSize")
        })?;
        let key_and_value_len =
            usize::try_from(read_u32_le(context, cursor)?).map_err(|_| {
                parse_error_value(
                    context,
                    format!(
                        "ktx key/value metadata record {record_index} keyAndValueByteSize overflows usize"
                    ),
                )
            })?;
        if key_and_value_len == 0 {
            return parse_error(
                context,
                format!(
                    "ktx key/value metadata record {record_index} keyAndValueByteSize must be nonzero"
                ),
            );
        }

        let payload_end = size_end.checked_add(key_and_value_len).ok_or_else(|| {
            parse_error_value(
                context,
                format!(
                    "ktx key/value metadata record {record_index} payload range overflows usize"
                ),
            )
        })?;
        require_metadata_record_range(context, payload_end, metadata_end, || {
            format!("ktx key/value metadata record {record_index} payload")
        })?;
        validate_key_value_metadata_key(context, record_index, size_end, payload_end)?;
        let padded_record_end = payload_end
            .checked_add(ktx_four_byte_padding(key_and_value_len))
            .ok_or_else(|| {
                parse_error_value(
                    context,
                    format!(
                        "ktx key/value metadata record {record_index} payload padded range overflows usize"
                    ),
                )
            })?;
        require_metadata_record_range(context, padded_record_end, metadata_end, || {
            format!("ktx key/value metadata record {record_index} payload padding")
        })?;
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
    label: impl FnOnce() -> String,
) -> Result<(), AssetImportError> {
    if required > metadata_end {
        let label = label();
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
        let image_size_end = cursor.checked_add(4).ok_or_else(|| {
            parse_error_value(
                context,
                format!("{} overflows usize", level_image_size_label(level_index)),
            )
        })?;
        require_len_lazy(context, image_size_end, || {
            level_image_size_label(level_index)
        })?;
        let image_size = usize::try_from(read_u32_le(context, cursor)?).map_err(|_| {
            parse_error_value(
                context,
                format!("{} overflows usize", level_image_size_label(level_index)),
            )
        })?;

        let payload_end = image_size_end.checked_add(image_size).ok_or_else(|| {
            parse_error_value(
                context,
                format!("{} range overflows usize", level_payload_label(level_index)),
            )
        })?;
        if image_size > 0 {
            require_len_lazy(context, payload_end, || level_payload_label(level_index))?;
        }
        cursor = payload_end;

        if level_index + 1 < mip_count {
            let padding_len = ktx_four_byte_padding(image_size);
            let padded_payload_end = cursor.checked_add(padding_len).ok_or_else(|| {
                parse_error_value(
                    context,
                    format!(
                        "{} padded range overflows usize",
                        level_payload_label(level_index)
                    ),
                )
            })?;
            require_len_lazy(context, padded_payload_end, || {
                format!("{} padding", level_payload_label(level_index))
            })?;
            if context.source_bytes[cursor..padded_payload_end]
                .iter()
                .any(|byte| *byte != 0)
            {
                return parse_error(
                    context,
                    format!(
                        "{} padding bytes must be zero",
                        level_payload_label(level_index)
                    ),
                );
            }
            cursor = padded_payload_end;
        }
    }
    Ok(())
}

fn require_len_lazy(
    context: &AssetImportContext,
    required: usize,
    label: impl FnOnce() -> String,
) -> Result<(), AssetImportError> {
    if context.source_bytes.len() < required {
        let label = label();
        return parse_error(
            context,
            format!(
                "{label} requires at least {required} bytes, got {}",
                context.source_bytes.len()
            ),
        );
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

#[cfg(test)]
mod plugins07_ktx1_hotpath_tests {
    use std::{hint::black_box, time::Instant};

    use super::*;
    use zircon_runtime::asset::AssetUri;

    const METADATA_RECORDS: usize = 64;
    const MIP_LEVELS: u32 = 16;

    fn context(bytes: Vec<u8>) -> AssetImportContext {
        AssetImportContext::new(
            "hotpath.ktx".into(),
            AssetUri::parse("res://textures/hotpath.ktx").expect("valid asset URI"),
            bytes,
            "".parse().expect("valid default texture settings"),
        )
    }

    fn metadata_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        for record_index in 0..METADATA_RECORDS {
            let payload = format!("key{record_index}\0value");
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(payload.as_bytes());
            bytes.resize(bytes.len() + ktx_four_byte_padding(payload.len()), 0);
        }
        bytes
    }

    fn mip_bytes() -> Vec<u8> {
        let mut bytes = Vec::with_capacity(MIP_LEVELS as usize * 8);
        for level_index in 0..MIP_LEVELS {
            bytes.extend_from_slice(&4_u32.to_le_bytes());
            bytes.extend_from_slice(&level_index.to_le_bytes());
        }
        bytes
    }

    fn legacy_require_metadata_record_range(
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

    fn legacy_validate_key_value_metadata_records(
        context: &AssetImportContext,
        metadata_start: usize,
        metadata_end: usize,
    ) -> Result<(), AssetImportError> {
        let mut cursor = metadata_start;
        let mut record_index = 0_usize;
        while cursor < metadata_end {
            let size_label =
                format!("ktx key/value metadata record {record_index} keyAndValueByteSize");
            let size_end = cursor.checked_add(4).ok_or_else(|| {
                parse_error_value(context, format!("{size_label} overflows usize"))
            })?;
            legacy_require_metadata_record_range(context, size_end, metadata_end, &size_label)?;
            let key_and_value_len = usize::try_from(read_u32_le(context, cursor)?)
                .map_err(|_| parse_error_value(context, format!("{size_label} overflows usize")))?;
            if key_and_value_len == 0 {
                return parse_error(context, format!("{size_label} must be nonzero"));
            }

            let payload_label = format!("ktx key/value metadata record {record_index} payload");
            let payload_end = size_end.checked_add(key_and_value_len).ok_or_else(|| {
                parse_error_value(context, format!("{payload_label} range overflows usize"))
            })?;
            legacy_require_metadata_record_range(
                context,
                payload_end,
                metadata_end,
                &payload_label,
            )?;
            validate_key_value_metadata_key(context, record_index, size_end, payload_end)?;
            let padded_record_end = payload_end
                .checked_add(ktx_four_byte_padding(key_and_value_len))
                .ok_or_else(|| {
                    parse_error_value(
                        context,
                        format!("{payload_label} padded range overflows usize"),
                    )
                })?;
            legacy_require_metadata_record_range(
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
                    format!(
                        "ktx key/value metadata record {record_index} padding bytes must be zero"
                    ),
                );
            }
            cursor = padded_record_end;
            record_index += 1;
        }
        Ok(())
    }

    fn legacy_validate_level_ranges(
        context: &AssetImportContext,
        first_level_offset: usize,
        mip_count: u32,
    ) -> Result<(), AssetImportError> {
        let mut cursor = first_level_offset;
        for level_index in 0..mip_count {
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

    #[test]
    fn plugins07_ktx1_hotpath_lazy_metadata_labels_preserve_validation() {
        let bytes = metadata_bytes();
        let context = context(bytes);
        validate_key_value_metadata_records(&context, 0, context.source_bytes.len())
            .expect("valid metadata records should pass");

        let truncated = context(vec![0; 2]);
        let error = validate_key_value_metadata_records(&truncated, 0, 2)
            .expect_err("truncated metadata size must fail closed")
            .to_string();
        assert!(
            error.contains(
                "ktx key/value metadata record 0 keyAndValueByteSize extends past declared ktx key/value metadata length"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn plugins07_ktx1_hotpath_lazy_mip_labels_preserve_validation() {
        let context = context(mip_bytes());
        validate_level_ranges(&context, 0, MIP_LEVELS).expect("valid mip level ranges should pass");

        let truncated = context(vec![0; 2]);
        let error = validate_level_ranges(&truncated, 0, 1)
            .expect_err("truncated mip imageSize must fail closed")
            .to_string();
        assert!(
            error.contains("ktx first mip level imageSize requires at least 4 bytes, got 2"),
            "unexpected error: {error}"
        );
    }

    #[test]
    #[ignore = "release-only lazy KTX1 metadata label benchmark"]
    fn plugins07_ktx1_hotpath_release_lazy_metadata_labels_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 2_500;
        let context = context(metadata_bytes());

        let (legacy_samples, optimized_samples) = alternating_samples(
            SAMPLE_PAIRS,
            || {
                measure(CHECKS_PER_SAMPLE, || {
                    legacy_validate_key_value_metadata_records(
                        black_box(&context),
                        0,
                        context.source_bytes.len(),
                    )
                })
            },
            || {
                measure(CHECKS_PER_SAMPLE, || {
                    validate_key_value_metadata_records(
                        black_box(&context),
                        0,
                        context.source_bytes.len(),
                    )
                })
            },
        );
        report_and_assert(
            "plugins07_ktx1_lazy_metadata_labels",
            SAMPLE_PAIRS,
            CHECKS_PER_SAMPLE,
            METADATA_RECORDS,
            CHECKS_PER_SAMPLE * METADATA_RECORDS * 3,
            &legacy_samples,
            &optimized_samples,
        );
    }

    #[test]
    #[ignore = "release-only lazy KTX1 mip label benchmark"]
    fn plugins07_ktx1_hotpath_release_lazy_mip_labels_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 5_000;
        let context = context(mip_bytes());

        let (legacy_samples, optimized_samples) = alternating_samples(
            SAMPLE_PAIRS,
            || {
                measure(CHECKS_PER_SAMPLE, || {
                    legacy_validate_level_ranges(black_box(&context), 0, MIP_LEVELS)
                })
            },
            || {
                measure(CHECKS_PER_SAMPLE, || {
                    validate_level_ranges(black_box(&context), 0, MIP_LEVELS)
                })
            },
        );
        report_and_assert(
            "plugins07_ktx1_lazy_mip_labels",
            SAMPLE_PAIRS,
            CHECKS_PER_SAMPLE,
            MIP_LEVELS as usize,
            CHECKS_PER_SAMPLE * (MIP_LEVELS as usize * 3 - 1),
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn measure(
        checks_per_sample: usize,
        mut validate: impl FnMut() -> Result<(), AssetImportError>,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..checks_per_sample {
            black_box(validate()).expect("benchmark fixture remains valid");
        }
        started.elapsed().as_nanos().max(1)
    }

    fn alternating_samples(
        sample_pairs: usize,
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) -> (Vec<u128>, Vec<u128>) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(sample_pairs);
        let mut optimized_samples = Vec::with_capacity(sample_pairs);
        for pair in 0..sample_pairs {
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn report_and_assert(
        name: &str,
        sample_pairs: usize,
        checks_per_sample: usize,
        items_per_check: usize,
        legacy_owned_labels_per_sample: usize,
        legacy_samples: &[u128],
        optimized_samples: &[u128],
    ) {
        let legacy_p95_ns = percentile(legacy_samples, 95);
        let optimized_p95_ns = percentile(optimized_samples, 95);
        let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
        println!(
            "PERF_RESULT {name} sample_pairs={sample_pairs} \
checks_per_sample={checks_per_sample} items_per_check={items_per_check} \
order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_owned_labels_per_sample={legacy_owned_labels_per_sample} \
optimized_owned_labels_per_sample=0 legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} \
threshold_percent=50 legacy_ns={} optimized_ns={}",
            raw(legacy_samples),
            raw(optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
            "lazy KTX1 labels must reduce P95 by at least 50%: \
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
