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
