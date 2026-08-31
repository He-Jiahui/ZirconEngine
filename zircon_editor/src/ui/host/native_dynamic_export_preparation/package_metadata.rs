use zircon_runtime::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

pub(super) fn module_crate(
    package: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Option<String> {
    package
        .modules
        .iter()
        .find(|module| module.kind == kind)
        .map(|module| module.crate_name.clone())
}

pub(super) fn sanitize_path_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        sanitized.push(if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            ch
        } else {
            '_'
        });
    }
    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::sanitize_path_component;

    #[test]
    fn optimization_batch_ej_path_sanitizer_preserves_ascii_and_replaces_other_chars() {
        assert_eq!(
            sanitize_path_component("package-name_42"),
            "package-name_42"
        );
        assert_eq!(
            sanitize_path_component("hello world/path"),
            "hello_world_path"
        );
        assert_eq!(sanitize_path_component(""), "_");
    }

    #[test]
    fn optimization_batch_ej_path_sanitizer_reserves_input_byte_upper_bound() {
        let source = include_str!("package_metadata.rs");
        let implementation = source
            .split("fn sanitize_path_component")
            .nth(1)
            .expect("path sanitizer implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("bounded production implementation");

        assert!(implementation.contains("String::with_capacity(value.len())"));
        assert!(implementation.contains("sanitized.push("));
        assert!(!implementation.contains(".collect()"));
    }

    #[test]
    #[ignore = "release-only preallocated package path sanitize benchmark"]
    fn optimization_batch_ej_preallocated_package_path_sanitize_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const SANITIZATIONS_PER_SAMPLE: usize = 2_048;

        fn legacy(value: &str) -> String {
            value
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                        ch
                    } else {
                        '_'
                    }
                })
                .collect()
        }

        fn measure_legacy(value: &str) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..SANITIZATIONS_PER_SAMPLE {
                let sanitized = black_box(legacy(black_box(value)));
                checksum = checksum.wrapping_add(sanitized.len());
                black_box(sanitized);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(value: &str) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..SANITIZATIONS_PER_SAMPLE {
                let sanitized = black_box(sanitize_path_component(black_box(value)));
                checksum = checksum.wrapping_add(sanitized.len());
                black_box(sanitized);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let fixture = format!("{}shipping target/output", "package-name_42/".repeat(256));
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&fixture));
                optimized_samples.push(measure_optimized(&fixture));
            } else {
                optimized_samples.push(measure_optimized(&fixture));
                legacy_samples.push(measure_legacy(&fixture));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR372_PREALLOCATED_PACKAGE_PATH_SANITIZE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             sanitizations_per_sample={SANITIZATIONS_PER_SAMPLE} input_bytes={} \
             pair_order=alternating_legacy_even legacy_initial_capacity=chars_lower_bound \
             optimized_initial_capacity={} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            fixture.len(),
            fixture.len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "preallocated package path sanitization must reduce P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
