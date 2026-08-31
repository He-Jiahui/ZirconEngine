pub(super) fn build_export_target_id(
    platform_id: &str,
    profile_name: &str,
    duplicate_platform: bool,
) -> String {
    if duplicate_platform {
        let mut target = String::with_capacity(platform_id.len() + 1 + profile_name.len());
        target.push_str(platform_id);
        target.push('.');
        push_build_export_key(&mut target, profile_name);
        target
    } else {
        platform_id.to_string()
    }
}

pub(super) fn build_export_key(value: &str) -> String {
    let mut key = String::with_capacity(value.len());
    push_build_export_key(&mut key, value);
    key
}

fn push_build_export_key(target: &mut String, value: &str) {
    let segment_start = target.len();
    let mut started = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            target.push(ch.to_ascii_lowercase());
            started = true;
        } else if started {
            target.push('_');
        }
    }
    while target.len() > segment_start && target.ends_with('_') {
        target.pop();
    }
    if target.len() == segment_start {
        target.push_str("target");
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_20260831gq_editor572_key_compatibility() {
        assert_eq!(build_export_key("  Desktop--Windows  "), "desktop__windows");
        assert_eq!(build_export_key("---"), "target");
    }

    #[test]
    fn optimization_batch_20260831gq_editor572_build_export_key_appends_to_an_existing_target_buffer()
     {
        let mut target = "windows.".to_owned();
        push_build_export_key(&mut target, "  Shipping--DX12  ");
        assert_eq!(target, "windows.shipping__dx12");

        let mut fallback = "linux.".to_owned();
        push_build_export_key(&mut fallback, "---");
        assert_eq!(fallback, "linux.target");
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260831gq_editor572_build_export_target_single_buffer_benchmark() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 200_000;
        const PLATFORM: &str = "windows-x86_64";
        const PROFILE: &str = "  Shipping--DX12 Validation  ";

        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut checksum = 0usize;
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let (elapsed, value) =
                    measure(ITERATIONS, PLATFORM, PROFILE, legacy_build_export_target_id);
                legacy_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) =
                    measure(ITERATIONS, PLATFORM, PROFILE, build_export_target_id);
                optimized_ns.push(elapsed);
                checksum ^= value;
            } else {
                let (elapsed, value) =
                    measure(ITERATIONS, PLATFORM, PROFILE, build_export_target_id);
                optimized_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) =
                    measure(ITERATIONS, PLATFORM, PROFILE, legacy_build_export_target_id);
                legacy_ns.push(elapsed);
                checksum ^= value;
            }
        }

        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
            "single-buffer P95 must be at least 25% below the two-allocation path: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "EDITOR572_BUILD_EXPORT_TARGET_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );

        fn measure(
            iterations: usize,
            platform: &str,
            profile: &str,
            operation: fn(&str, &str, bool) -> String,
        ) -> (u128, usize) {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..iterations {
                checksum = checksum
                    .wrapping_add(operation(black_box(platform), black_box(profile), true).len());
            }
            (started.elapsed().as_nanos(), black_box(checksum))
        }
    }

    fn legacy_build_export_target_id(
        platform_id: &str,
        profile_name: &str,
        duplicate_platform: bool,
    ) -> String {
        if duplicate_platform {
            format!("{platform_id}.{}", build_export_key(profile_name))
        } else {
            platform_id.to_string()
        }
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = (ordered.len() * percentile).div_ceil(100).max(1);
        ordered[rank - 1]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
