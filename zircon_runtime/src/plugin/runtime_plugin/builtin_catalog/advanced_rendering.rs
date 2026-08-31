use crate::{plugin::CapabilityStatus, plugin::PluginMaturity};

use super::capability_status::capability_status;
use super::BuiltinCatalogDescriptorBuilder;

pub(super) fn is_advanced_render_descriptor(package_id: &str) -> bool {
    matches!(package_id, "virtual_geometry" | "hybrid_gi" | "solari")
}

pub(super) fn classify_advanced_render_descriptor(
    package_id: &str,
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    match package_id {
        "virtual_geometry" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability("runtime.render.advanced.virtual_geometry")
            .with_capability_status(capability_status(
                "runtime.plugin.virtual_geometry",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(
                capability_status(
                    "runtime.render.advanced.virtual_geometry",
                    CapabilityStatus::Partial,
                )
                .with_note(
                    "AdvancedRender provider path; default render profiles do not require it.",
                ),
            ),
        "hybrid_gi" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability("runtime.render.advanced.hybrid_gi")
            .with_capability_status(capability_status(
                "runtime.plugin.hybrid_gi",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(
                capability_status(
                    "runtime.render.advanced.hybrid_gi",
                    CapabilityStatus::Partial,
                )
                .with_note(
                    "AdvancedRender provider path; default render profiles do not require it.",
                ),
            ),
        "solari" => descriptor
            .with_maturity(PluginMaturity::Experimental)
            .with_capability("runtime.render.experimental.solari")
            .with_capability_status(capability_status(
                "runtime.plugin.solari",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(
                capability_status(
                    "runtime.render.experimental.solari",
                    CapabilityStatus::Partial,
                )
                .with_note(
                    "Solari realtime raytraced lighting pass executor is not implemented yet",
                ),
            ),
        _ => descriptor,
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::is_advanced_render_descriptor;

    const CHECKS_PER_SAMPLE: usize = 1_048_576;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_gc_runtime485_advanced_render_dispatch_preserves_package_set() {
        for package_id in ["virtual_geometry", "hybrid_gi", "solari"] {
            assert!(is_advanced_render_descriptor(package_id), "{package_id}");
        }
        for package_id in ["", "rendering", "Solari", "solari.preview"] {
            assert!(!is_advanced_render_descriptor(package_id), "{package_id}");
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_gc_runtime485_advanced_render_dispatch_benchmark() {
        const INPUT: &str = "solari";
        for _ in 0..4 {
            black_box(measure_checks(INPUT, false));
            black_box(measure_checks(INPUT, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(INPUT, false));
                optimized_samples.push(measure_checks(INPUT, true));
            } else {
                optimized_samples.push(measure_checks(INPUT, true));
                legacy_samples.push(measure_checks(INPUT, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME485_ADVANCED_RENDER_ID_DISPATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_candidate_comparisons_per_check=3 optimized_static_dispatch_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            INPUT.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let matched = if optimized {
                is_advanced_render_descriptor(black_box(input))
            } else {
                ["virtual_geometry", "hybrid_gi", "solari"].contains(&black_box(input))
            };
            black_box(matched);
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
