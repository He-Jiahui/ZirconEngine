use crate::plugin::RuntimeExtensionRegistry;

use super::super::RuntimePluginRegistrationReport;
use super::super::extension_merge::merge_runtime_extensions;
use super::super::registration::order::order_runtime_plugin_registration_reports;
use super::RuntimeExtensionCatalogReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn runtime_extension_report(
    registrations: &[RuntimePluginRegistrationReport],
) -> RuntimeExtensionCatalogReport {
    let mut registry = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::with_capacity(registrations.len());
    let mut fatal_diagnostics = Vec::with_capacity(registrations.len());
    let registrations = match order_runtime_plugin_registration_reports(registrations) {
        Ok(registrations) => registrations,
        Err(error) => {
            let diagnostic = format!("runtime plugin module descriptor ordering failed: {error}");
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
            registry.finalize();
            return RuntimeExtensionCatalogReport {
                registry,
                diagnostics,
                fatal_diagnostics,
            };
        }
    };
    for registration in registrations {
        merge_runtime_extensions(
            registration,
            &mut registry,
            &mut diagnostics,
            &mut fatal_diagnostics,
        );
    }
    registry.finalize();
    RuntimeExtensionCatalogReport {
        registry,
        diagnostics,
        fatal_diagnostics,
    }
}

#[cfg(test)]
mod optimization_batch_20260830br_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const DIAGNOSTICS_PER_SAMPLE: usize = 1_024;

    #[test]
    fn extension_report_reserves_registration_diagnostic_upper_bounds() {
        let source = include_str!("runtime.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(registrations.len())"));
        assert_eq!(
            implementation
                .matches("Vec::with_capacity(registrations.len())")
                .count(),
            2
        );
    }

    #[test]
    fn extension_report_keeps_registration_merge_order_after_reserving() {
        let source = include_str!("runtime.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let capacity = implementation
            .find("Vec::with_capacity(registrations.len())")
            .expect("diagnostic capacity reservation");
        let merge = implementation
            .find("for registration in registrations")
            .expect("registration merge loop");
        assert!(capacity < merge);
        assert!(implementation.contains("merge_runtime_extensions("));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830br_runtime_extension_report_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME370_EXTENSION_REPORT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} diagnostics_per_sample={DIAGNOSTICS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut diagnostics = if optimized {
                Vec::with_capacity(DIAGNOSTICS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..DIAGNOSTICS_PER_SAMPLE {
                diagnostics.push(index);
            }
            checksum ^= diagnostics.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
