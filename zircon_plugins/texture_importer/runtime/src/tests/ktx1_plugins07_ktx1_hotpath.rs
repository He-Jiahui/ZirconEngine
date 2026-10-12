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
