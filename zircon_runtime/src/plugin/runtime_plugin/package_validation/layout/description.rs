use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_description(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.description.is_empty()
        && has_outer_whitespace(&package_manifest.description)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest description `{}` must be trimmed when present",
            package_manifest.description
        ));
    }
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{has_outer_whitespace, validate_runtime_plugin_package_description};
    use crate::plugin::PluginPackageManifest;

    const DESCRIPTION_BYTES: usize = 4_096;
    const CHECKS_PER_SAMPLE: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fx_runtime480_description_edge_check_preserves_trim_semantics() {
        assert!(!has_outer_whitespace(""));
        assert!(!has_outer_whitespace("runtime package"));
        assert!(has_outer_whitespace(" runtime package"));
        assert!(has_outer_whitespace("runtime package\u{2003}"));

        let mut manifest = PluginPackageManifest::new("runtime", "Runtime");
        manifest.description = "runtime package\u{2003}".to_string();
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_package_description(&manifest, &mut diagnostics);
        assert_eq!(diagnostics.len(), 1);

        manifest.description = "runtime package".to_string();
        diagnostics.clear();
        validate_runtime_plugin_package_description(&manifest, &mut diagnostics);
        assert!(diagnostics.is_empty());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fx_runtime480_description_edge_check_benchmark() {
        let mut description = String::with_capacity(DESCRIPTION_BYTES + 1);
        description.push('x');
        description.extend(std::iter::repeat_n(' ', DESCRIPTION_BYTES));
        for _ in 0..4 {
            black_box(measure_checks(&description, false));
            black_box(measure_checks(&description, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(&description, false));
                optimized_samples.push(measure_checks(&description, true));
            } else {
                optimized_samples.push(measure_checks(&description, true));
                legacy_samples.push(measure_checks(&description, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME480_PACKAGE_DESCRIPTION_EDGE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} description_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_trimmed_bytes_per_check={DESCRIPTION_BYTES} optimized_edge_scalars_per_check=2 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
            description.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 25 / 100);
    }

    fn measure_checks(description: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let has_outer = if optimized {
                has_outer_whitespace(black_box(description))
            } else {
                let value = black_box(description);
                value.trim() != value
            };
            black_box(has_outer);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
