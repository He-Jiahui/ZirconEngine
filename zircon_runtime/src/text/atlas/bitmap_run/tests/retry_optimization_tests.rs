#[test]
fn optimization_batch_20260830cw_bitmap_retry_capacity_source_contract() {
    let source = include_str!("../retry.rs");
    let input = source
        .split("let frame_sources = frame_sources.into_iter()")
        .nth(1)
        .expect("bitmap retry input capacity implementation")
        .split("for glyph in retry_plan.retry_glyphs")
        .next()
        .expect("bounded bitmap retry input capacity implementation");

    assert!(input.contains("HashSet::with_capacity(retry_source_count)"));
    assert!(input.contains("Vec::with_capacity(source_capacity)"));
    assert!(input.contains("minimum_frame_source_count"));
    assert!(!input.contains("collect::<HashSet"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260830cw_runtime_bitmap_retry_capacity_p95() {
    fn measure(keys: &[Option<u64>], reserve: bool) -> u128 {
        let started = std::time::Instant::now();
        for _ in 0..32 {
            let mut scheduled = if reserve {
                std::collections::HashSet::with_capacity(keys.len())
            } else {
                std::collections::HashSet::new()
            };
            scheduled.extend(std::hint::black_box(keys).iter().filter_map(|key| *key));
            let mut sources = if reserve {
                Vec::with_capacity(keys.len().saturating_mul(2))
            } else {
                Vec::new()
            };
            let mut origins = if reserve {
                Vec::with_capacity(keys.len().saturating_mul(2))
            } else {
                Vec::new()
            };
            for (index, key) in keys.iter().enumerate() {
                if key.is_some() {
                    sources.push(index as u64);
                    origins.push(index);
                }
            }
            std::hint::black_box((scheduled, sources, origins));
        }
        started.elapsed().as_nanos()
    }

    let keys = (0..32_768_u64)
        .map(|index| (index % 8 != 0).then_some(index % 16_384))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for sample_index in 0..17 {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&keys, false));
            optimized_samples.push(measure(&keys, true));
        } else {
            optimized_samples.push(measure(&keys, true));
            legacy_samples.push(measure(&keys, false));
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "RUNTIME399_BITMAP_RETRY_CAPACITY_BENCH_V1 retry_sources={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
        keys.len(),
        legacy_p95,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "capacity-sized bitmap retry P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}
